# Snake Game in Rust and WebAssembly

This project provides a simple implementation of the classic Snake game
written in Rust and compiled to WebAssembly. The game renders on an HTML
`<canvas>` element and is controlled with the arrow keys.

## Building

Install the required target and build the WASM package:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build --target web
```

## Running

After building, an output directory `pkg/` is created. Serve the project
root with any static web server and open `index.html` in a browser:

```bash
python3 -m http.server
```

Navigate to `http://localhost:8000` to play.

## Deploying

Copy the `index.html` and `pkg/` directory to any static hosting service
such as GitHub Pages or your own web server.
