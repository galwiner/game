use std::cell::RefCell;
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent, HtmlElement,
};

const WIDTH: i32 = 20;
const HEIGHT: i32 = 20;
const DEPTH: i32 = 20;
const CELL: f64 = 20.0;

fn set_score(score: i32) {
    let window = web_sys::window().unwrap();
    if let Some(document) = window.document() {
        if let Some(elem) = document.get_element_by_id("score") {
            elem.set_text_content(Some(&format!("Score: {}", score)));
        }
    }
}

fn show_restart(show: bool) {
    let window = web_sys::window().unwrap();
    if let Some(document) = window.document() {
        if let Some(elem) = document.get_element_by_id("restart") {
            let html: HtmlElement = elem.dyn_into().unwrap();
            html.style()
                .set_property("display", if show { "inline-block" } else { "none" })
                .unwrap();
        }
    }
}

thread_local! {
    static GAME: RefCell<Option<GameVariant>> = RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas: HtmlCanvasElement = document.get_element_by_id("game").unwrap().dyn_into()?;
    canvas.set_width((WIDTH as f64 * CELL) as u32);
    canvas.set_height((HEIGHT as f64 * CELL) as u32);
    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    GAME.with(|g| g.borrow_mut().replace(GameVariant::TwoD(Game2D::new(ctx))));

    // keyboard events
    {
        let doc = document.clone();
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            event.prevent_default();
            let key = event.key();
            GAME.with(|game| {
                if let Some(g) = game.borrow_mut().as_mut() {
                    g.change_dir(&key);
                }
            });
        }) as Box<dyn FnMut(_)>);
        doc.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // game loop
    {
        let closure = Closure::wrap(Box::new(move || {
            GAME.with(|game| {
                if let Some(g) = game.borrow_mut().as_mut() {
                    g.update();
                    g.draw().unwrap();
                }
            });
        }) as Box<dyn FnMut()>);
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            100,
        )?;
        closure.forget();
    }
    Ok(())
}

#[wasm_bindgen]
pub fn toggle_mode() {
    GAME.with(|g| {
        let mut game = g.borrow_mut();
        if let Some(current) = game.take() {
            let new_game = match current {
                GameVariant::TwoD(g2d) => {
                    let ctx = g2d.ctx.clone();
                    GameVariant::ThreeD(Game3D::new(ctx))
                }
                GameVariant::ThreeD(g3d) => {
                    let ctx = g3d.ctx.clone();
                    GameVariant::TwoD(Game2D::new(ctx))
                }
            };
            game.replace(new_game);
        }
    });
}

#[wasm_bindgen]
pub fn restart() {
    GAME.with(|g| {
        if let Some(game) = g.borrow_mut().as_mut() {
            game.restart();
        }
    });
    show_restart(false);
}

enum GameVariant {
    TwoD(Game2D),
    ThreeD(Game3D),
}

impl GameVariant {
    fn change_dir(&mut self, key: &str) {
        match self {
            GameVariant::TwoD(g) => g.change_dir(key),
            GameVariant::ThreeD(g) => g.change_dir(key),
        }
    }
    fn update(&mut self) {
        match self {
            GameVariant::TwoD(g) => g.update(),
            GameVariant::ThreeD(g) => g.update(),
        }
    }
    fn draw(&self) -> Result<(), JsValue> {
        match self {
            GameVariant::TwoD(g) => g.draw(),
            GameVariant::ThreeD(g) => g.draw(),
        }
    }
    fn restart(&mut self) {
        match self {
            GameVariant::TwoD(g) => {
                let ctx = g.ctx.clone();
                *g = Game2D::new(ctx);
            }
            GameVariant::ThreeD(g) => {
                let ctx = g.ctx.clone();
                *g = Game3D::new(ctx);
            }
        }
    }
}

struct Game2D {
    ctx: CanvasRenderingContext2d,
    snake: VecDeque<(i32, i32)>,
    dir: (i32, i32),
    food: (i32, i32),
    alive: bool,
    score: i32,
}

impl Game2D {
    fn new(ctx: CanvasRenderingContext2d) -> Self {
        let mut snake = VecDeque::new();
        snake.push_back((WIDTH / 2, HEIGHT / 2));
        let food = (5, 5);
        set_score(0);
        show_restart(false);
        Self {
            ctx,
            snake,
            dir: (1, 0),
            food,
            alive: true,
            score: 0,
        }
    }

    fn change_dir(&mut self, key: &str) {
        match key {
            "ArrowUp" if self.dir.1 != 1 => self.dir = (0, -1),
            "ArrowDown" if self.dir.1 != -1 => self.dir = (0, 1),
            "ArrowLeft" if self.dir.0 != 1 => self.dir = (-1, 0),
            "ArrowRight" if self.dir.0 != -1 => self.dir = (1, 0),
            _ => {}
        }
    }

    fn update(&mut self) {
        if !self.alive {
            return;
        }
        let mut new_head = *self.snake.front().unwrap();
        new_head.0 = (new_head.0 + self.dir.0 + WIDTH) % WIDTH;
        new_head.1 = (new_head.1 + self.dir.1 + HEIGHT) % HEIGHT;
        if self.snake.contains(&new_head) {
            self.alive = false;
            show_restart(true);
            return;
        }
        self.snake.push_front(new_head);
        if new_head == self.food {
            self.score += 1;
            set_score(self.score);
            self.food = (
                (js_sys::Math::random() * WIDTH as f64) as i32,
                (js_sys::Math::random() * HEIGHT as f64) as i32,
            );
        } else {
            self.snake.pop_back();
        }
    }

    fn draw(&self) -> Result<(), JsValue> {
        self.ctx.set_fill_style(&JsValue::from_str("black"));
        self.ctx
            .fill_rect(0.0, 0.0, WIDTH as f64 * CELL, HEIGHT as f64 * CELL);
        self.ctx.set_fill_style(&JsValue::from_str("green"));
        for (x, y) in self.snake.iter() {
            self.ctx
                .fill_rect(*x as f64 * CELL, *y as f64 * CELL, CELL, CELL);
        }
        self.ctx.set_fill_style(&JsValue::from_str("red"));
        self.ctx.fill_rect(
            self.food.0 as f64 * CELL,
            self.food.1 as f64 * CELL,
            CELL,
            CELL,
        );
        set_score(self.score);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Vec3(i32, i32, i32);

impl Vec3 {
    fn add(&self, other: Vec3) -> Vec3 {
        Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
    fn wrap(&self) -> Vec3 {
        Vec3(
            (self.0 + WIDTH) % WIDTH,
            (self.1 + HEIGHT) % HEIGHT,
            (self.2 + DEPTH) % DEPTH,
        )
    }
    fn neg(&self) -> Vec3 {
        Vec3(-self.0, -self.1, -self.2)
    }
}

struct Orientation {
    f: Vec3,
    u: Vec3,
    r: Vec3,
}

impl Orientation {
    fn new() -> Self {
        Self {
            f: Vec3(0, 0, 1),
            u: Vec3(1, 0, 0),
            r: Vec3(0, 1, 0),
        }
    }
    fn pitch_up(&mut self) {
        let new_f = self.u;
        self.u = self.f.neg();
        self.f = new_f;
    }
    fn pitch_down(&mut self) {
        let new_f = self.u.neg();
        self.u = self.f;
        self.f = new_f;
    }
    fn yaw_left(&mut self) {
        let new_f = self.r.neg();
        self.r = self.f;
        self.f = new_f;
    }
    fn yaw_right(&mut self) {
        let new_f = self.r;
        self.r = self.f.neg();
        self.f = new_f;
    }
}

struct Game3D {
    ctx: CanvasRenderingContext2d,
    snake: VecDeque<Vec3>,
    orient: Orientation,
    food: Vec3,
    alive: bool,
    score: i32,
}

impl Game3D {
    fn new(ctx: CanvasRenderingContext2d) -> Self {
        let mut snake = VecDeque::new();
        snake.push_back(Vec3(WIDTH / 2, HEIGHT / 2, DEPTH / 2));
        let food = Vec3(5, 5, 5);
        set_score(0);
        show_restart(false);
        Self {
            ctx,
            snake,
            orient: Orientation::new(),
            food,
            alive: true,
            score: 0,
        }
    }

    fn change_dir(&mut self, key: &str) {
        match key {
            "ArrowUp" => self.orient.pitch_up(),
            "ArrowDown" => self.orient.pitch_down(),
            "ArrowLeft" => self.orient.yaw_left(),
            "ArrowRight" => self.orient.yaw_right(),
            _ => {}
        }
    }

    fn update(&mut self) {
        if !self.alive {
            return;
        }
        let head = *self.snake.front().unwrap();
        let mut new_head = head.add(self.orient.f);
        new_head = new_head.wrap();
        if self.snake.contains(&new_head) {
            self.alive = false;
            show_restart(true);
            return;
        }
        if new_head.0 == self.food.0 && new_head.1 == self.food.1 && new_head.2 == self.food.2 {
            self.score += 1;
            set_score(self.score);
            self.food = Vec3(
                (js_sys::Math::random() * WIDTH as f64) as i32,
                (js_sys::Math::random() * HEIGHT as f64) as i32,
                (js_sys::Math::random() * DEPTH as f64) as i32,
            );
        } else {
            self.snake.pop_back();
        }
        self.snake.push_front(new_head);
    }

    fn draw(&self) -> Result<(), JsValue> {
        self.ctx.set_fill_style(&JsValue::from_str("black"));
        self.ctx.fill_rect(0.0, 0.0, WIDTH as f64 * CELL, HEIGHT as f64 * CELL);

        // draw from farthest to nearest for basic occlusion
        let mut items: Vec<(Vec3, &str)> = self
            .snake
            .iter()
            .map(|p| (*p, "green"))
            .collect();
        items.push((self.food, "red"));
        items.sort_by_key(|(p, _)| p.2);
        for (p, color) in items.into_iter() {
            draw_cube(&self.ctx, p, color);
        }

        set_score(self.score);
        Ok(())
    }
}

fn project_point(x: f64, y: f64, z: f64) -> (f64, f64) {
    let d = DEPTH as f64 * 2.0;
    let zf = z + d;
    let px = (x - WIDTH as f64 / 2.0) * d / zf + WIDTH as f64 / 2.0;
    let py = (y - HEIGHT as f64 / 2.0) * d / zf + HEIGHT as f64 / 2.0;
    (px * CELL, py * CELL)
}

fn draw_cube(ctx: &CanvasRenderingContext2d, pos: Vec3, color: &str) {
    let p000 = project_point(pos.0 as f64, pos.1 as f64, pos.2 as f64);
    let p100 = project_point(pos.0 as f64 + 1.0, pos.1 as f64, pos.2 as f64);
    let p010 = project_point(pos.0 as f64, pos.1 as f64 + 1.0, pos.2 as f64);
    let p110 = project_point(pos.0 as f64 + 1.0, pos.1 as f64 + 1.0, pos.2 as f64);
    let p001 = project_point(pos.0 as f64, pos.1 as f64, pos.2 as f64 + 1.0);
    let p101 = project_point(pos.0 as f64 + 1.0, pos.1 as f64, pos.2 as f64 + 1.0);
    let p011 = project_point(pos.0 as f64, pos.1 as f64 + 1.0, pos.2 as f64 + 1.0);
    let p111 = project_point(pos.0 as f64 + 1.0, pos.1 as f64 + 1.0, pos.2 as f64 + 1.0);

    // back face
    ctx.set_fill_style(&JsValue::from_str(color));
    ctx.set_global_alpha(0.2);
    ctx.begin_path();
    ctx.move_to(p001.0, p001.1);
    ctx.line_to(p101.0, p101.1);
    ctx.line_to(p111.0, p111.1);
    ctx.line_to(p011.0, p011.1);
    ctx.close_path();
    ctx.fill();

    // top face
    ctx.set_global_alpha(0.6);
    ctx.begin_path();
    ctx.move_to(p011.0, p011.1);
    ctx.line_to(p111.0, p111.1);
    ctx.line_to(p110.0, p110.1);
    ctx.line_to(p010.0, p010.1);
    ctx.close_path();
    ctx.fill();

    // right face
    ctx.set_global_alpha(0.4);
    ctx.begin_path();
    ctx.move_to(p101.0, p101.1);
    ctx.line_to(p111.0, p111.1);
    ctx.line_to(p110.0, p110.1);
    ctx.line_to(p100.0, p100.1);
    ctx.close_path();
    ctx.fill();

    // front face
    ctx.set_global_alpha(1.0);
    ctx.begin_path();
    ctx.move_to(p000.0, p000.1);
    ctx.line_to(p100.0, p100.1);
    ctx.line_to(p110.0, p110.1);
    ctx.line_to(p010.0, p010.1);
    ctx.close_path();
    ctx.fill();

    // edges
    ctx.begin_path();
    ctx.move_to(p000.0, p000.1);
    ctx.line_to(p001.0, p001.1);
    ctx.move_to(p100.0, p100.1);
    ctx.line_to(p101.0, p101.1);
    ctx.move_to(p110.0, p110.1);
    ctx.line_to(p111.0, p111.1);
    ctx.move_to(p010.0, p010.1);
    ctx.line_to(p011.0, p011.1);
    ctx.stroke();
}
