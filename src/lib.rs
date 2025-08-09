use std::cell::RefCell;
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    CanvasRenderingContext2d, HtmlButtonElement, HtmlCanvasElement, HtmlElement, KeyboardEvent,
};

const WIDTH: i32 = 20;
const HEIGHT: i32 = 20;
const CELL: f64 = 20.0;

thread_local! {
    static GAME: RefCell<Option<Game>> = RefCell::new(None);
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
    let score_el: HtmlElement = document.get_element_by_id("score").unwrap().dyn_into()?;
    let restart_btn: HtmlButtonElement =
        document.get_element_by_id("restart").unwrap().dyn_into()?;
    let game = Game::new(ctx.clone(), score_el.clone(), restart_btn.clone());
    GAME.with(|g| g.borrow_mut().replace(game));

    // restart button
    {
        let btn = restart_btn.clone();
        let closure = Closure::wrap(Box::new(move || {
            GAME.with(|game| {
                if let Some(g) = game.borrow_mut().as_mut() {
                    g.reset();
                }
            });
        }) as Box<dyn FnMut()>);
        btn.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // keyboard events
    {
        let doc = document.clone();
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();
            GAME.with(|game| {
                if let Some(g) = game.borrow_mut().as_mut() {
                    if key == "Enter" {
                        if g.over {
                            g.reset();
                        }
                    } else {
                        g.change_dir(&key);
                    }
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

struct Game {
    ctx: CanvasRenderingContext2d,
    snake: VecDeque<(i32, i32)>,
    dir: (i32, i32),
    food: (i32, i32),
    score: i32,
    over: bool,
    score_el: HtmlElement,
    restart_btn: HtmlButtonElement,
}

impl Game {
    fn new(
        ctx: CanvasRenderingContext2d,
        score_el: HtmlElement,
        restart_btn: HtmlButtonElement,
    ) -> Self {
        let mut snake = VecDeque::new();
        snake.push_back((WIDTH / 2, HEIGHT / 2));
        let food = (
            (js_sys::Math::random() * WIDTH as f64) as i32,
            (js_sys::Math::random() * HEIGHT as f64) as i32,
        );
        let mut game = Self {
            ctx,
            snake,
            dir: (1, 0),
            food,
            score: 0,
            over: false,
            score_el,
            restart_btn,
        };
        game.update_score();
        let _ = game.restart_btn.style().set_property("display", "none");
        game
    }

    fn reset(&mut self) {
        self.snake.clear();
        self.snake.push_back((WIDTH / 2, HEIGHT / 2));
        self.dir = (1, 0);
        self.food = (
            (js_sys::Math::random() * WIDTH as f64) as i32,
            (js_sys::Math::random() * HEIGHT as f64) as i32,
        );
        self.score = 0;
        self.over = false;
        self.update_score();
        let _ = self.restart_btn.style().set_property("display", "none");
    }

    fn change_dir(&mut self, key: &str) {
        if self.over {
            return;
        }
        match key {
            "ArrowUp" if self.dir.1 != 1 => self.dir = (0, -1),
            "ArrowDown" if self.dir.1 != -1 => self.dir = (0, 1),
            "ArrowLeft" if self.dir.0 != 1 => self.dir = (-1, 0),
            "ArrowRight" if self.dir.0 != -1 => self.dir = (1, 0),
            _ => {}
        }
    }

    fn update(&mut self) {
        if self.over {
            return;
        }
        let mut new_head = *self.snake.front().unwrap();
        new_head.0 = (new_head.0 + self.dir.0 + WIDTH) % WIDTH;
        new_head.1 = (new_head.1 + self.dir.1 + HEIGHT) % HEIGHT;
        if self.snake.contains(&new_head) {
            self.over = true;
            let _ = self.restart_btn.style().set_property("display", "block");
            return;
        }
        if new_head == self.food {
            self.score += 1;
            self.update_score();
            self.food = (
                (js_sys::Math::random() * WIDTH as f64) as i32,
                (js_sys::Math::random() * HEIGHT as f64) as i32,
            );
        } else {
            self.snake.pop_back();
        }
        self.snake.push_front(new_head);
    }

    fn update_score(&self) {
        self.score_el
            .set_text_content(Some(&format!("Score: {}", self.score)));
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
        Ok(())
    }
}
