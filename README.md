# KineticGame
Actually the name will change, but the goal of this project is to make a playable game runing on browser (whatever browser btw) and it will be optimize as much as possible

## How to develop

If you want to have an instant rendering you need to install [trunk](https://crates.io/crates/trunk/0.5.1) and wasm target:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Then, launch real time renderer with:
```bash
trunk serve
```

And head to `http://localhost:8080` to test your game !