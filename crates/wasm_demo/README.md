# Rust Analyzer WASM demo

## building

```sh
cd crates/wasm_demo
wasm-pack build .

cd www
npm i
npx webpack-dev-server --host 0.0.0.0
```
