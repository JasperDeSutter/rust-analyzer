import "wasm_demo";
import * as monaco from "monaco-editor";

console.log('loading wasm done')

const code = `
fn main() {
    println!("Hello, world!");
}
`

monaco.editor.create(document.body, {
    value: code,
    language: "rust"
});
