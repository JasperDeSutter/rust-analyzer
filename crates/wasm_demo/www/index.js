// import 'monaco-editor/esm/vs/editor/browser/controller/coreCommands.js';
// import * as monaco from 'monaco-editor/esm/vs/editor/editor.api.js';
// import 'monaco-editor/esm/vs/basic-languages/rust/rust.contribution';

import * as monaco from 'monaco-editor';
import example_code from "./example-code"

import "./index.css";

self.MonacoEnvironment = {
    getWorkerUrl: () => "./editor.worker.bundle.js",
};

monaco.editor.create(document.body, {
    theme: 'vs-dark',
    value: example_code,
    language: "rust"
});

async function setupMode() {
    console.warn('setupMode')
    const { WorldState } = await import("wasm_demo")

    const models = monaco.editor.getModels()
    const state = new WorldState(models[0].getValue())

    monaco.languages.registerCompletionItemProvider("rust", {
        triggerCharacters: ["."],
        provideCompletionItems: (_, position) => state.on_dot_typed(position.lineNumber, position.column),
    });

    monaco.languages.registerHoverProvider("rust", {
        provideHover: (_, position) => state.hover(position.lineNumber, position.column)
    })
}

setupMode()
