// @ts-check
import "monaco-editor/esm/vs/editor/browser/controller/coreCommands";
import "monaco-editor/esm/vs/editor/browser/widget/codeEditorWidget";
import "monaco-editor/esm/vs/editor/browser/widget/diffEditorWidget";
import "monaco-editor/esm/vs/editor/browser/widget/diffNavigator";
import "monaco-editor/esm/vs/editor/contrib/bracketMatching/bracketMatching";
import "monaco-editor/esm/vs/editor/contrib/caretOperations/caretOperations";
import "monaco-editor/esm/vs/editor/contrib/caretOperations/transpose";
import "monaco-editor/esm/vs/editor/contrib/clipboard/clipboard";
import "monaco-editor/esm/vs/editor/contrib/codelens/codelensController";
import "monaco-editor/esm/vs/editor/contrib/colorPicker/colorDetector";
import "monaco-editor/esm/vs/editor/contrib/comment/comment";
import "monaco-editor/esm/vs/editor/contrib/contextmenu/contextmenu";
import "monaco-editor/esm/vs/editor/contrib/cursorUndo/cursorUndo";
import "monaco-editor/esm/vs/editor/contrib/dnd/dnd";
import "monaco-editor/esm/vs/editor/contrib/find/findController";
import "monaco-editor/esm/vs/editor/contrib/folding/folding";
import "monaco-editor/esm/vs/editor/contrib/format/formatActions";
import "monaco-editor/esm/vs/editor/contrib/goToDefinition/goToDefinitionCommands";
import "monaco-editor/esm/vs/editor/contrib/goToDefinition/goToDefinitionMouse";
import "monaco-editor/esm/vs/editor/contrib/gotoError/gotoError";
import "monaco-editor/esm/vs/editor/contrib/hover/hover";
import "monaco-editor/esm/vs/editor/contrib/inPlaceReplace/inPlaceReplace";
import "monaco-editor/esm/vs/editor/contrib/linesOperations/linesOperations";
import "monaco-editor/esm/vs/editor/contrib/links/links";
import "monaco-editor/esm/vs/editor/contrib/multicursor/multicursor";
import "monaco-editor/esm/vs/editor/contrib/parameterHints/parameterHints";
import "monaco-editor/esm/vs/editor/contrib/referenceSearch/referenceSearch";
import "monaco-editor/esm/vs/editor/contrib/rename/rename";
import "monaco-editor/esm/vs/editor/contrib/smartSelect/smartSelect";
import "monaco-editor/esm/vs/editor/contrib/snippet/snippetController2";
import "monaco-editor/esm/vs/editor/contrib/suggest/suggestController";
import "monaco-editor/esm/vs/editor/contrib/toggleTabFocusMode/toggleTabFocusMode";
import "monaco-editor/esm/vs/editor/contrib/wordHighlighter/wordHighlighter";
import "monaco-editor/esm/vs/editor/contrib/wordOperations/wordOperations";
import "monaco-editor/esm/vs/editor/standalone/browser/accessibilityHelp/accessibilityHelp";
import "monaco-editor/esm/vs/editor/standalone/browser/inspectTokens/inspectTokens";
import "monaco-editor/esm/vs/editor/standalone/browser/iPadShowKeyboard/iPadShowKeyboard";
import "monaco-editor/esm/vs/editor/standalone/browser/quickOpen/quickOutline";
import "monaco-editor/esm/vs/editor/standalone/browser/quickOpen/gotoLine";
import "monaco-editor/esm/vs/editor/standalone/browser/quickOpen/quickCommand";
import "monaco-editor/esm/vs/editor/standalone/browser/toggleHighContrast/toggleHighContrast";

import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import * as rust_conf from "monaco-editor/esm/vs/basic-languages/rust/rust";

import example_code from "./example-code";
const wasm_demo = import("wasm_demo");

import "./index.css";

self.MonacoEnvironment = {
    getWorkerUrl: () => "./editor.worker.bundle.js",
};

monaco.languages.register({
    id: "rust",
    extensions: [".rs", ".rlib"],
    aliases: ["Rust", "rust"],
});

monaco.languages.onLanguage("rust", async () => {
    const { WorldState } = await wasm_demo;

    const [model] = monaco.editor.getModels();
    const state = new WorldState();

    function update() {
        const res = state.update(model.getValue())
        monaco.editor.setModelMarkers(model, "rust", res.diagnostics)
    }
    update()

    let timeout = 0
    model.onDidChangeContent(e => {
        if (timeout != 0) {
            clearTimeout(timeout)
        }
        console.warn('update')
        timeout = setTimeout(update)
    })

    monaco.languages.setLanguageConfiguration("rust", rust_conf.conf);
    monaco.languages.setMonarchTokensProvider("rust", rust_conf.language);

    // This panics when typing:
    // monaco.languages.registerCompletionItemProvider("rust", {
    //     triggerCharacters: ["."],
    //     provideCompletionItems: (_, pos) => state.on_dot_typed(pos.lineNumber, pos.column),
    // });

    monaco.languages.registerHoverProvider("rust", {
        provideHover: (_, pos) => state.hover(pos.lineNumber, pos.column),
    });
    monaco.languages.registerCodeLensProvider("rust", {
        provideCodeLenses: () => state.code_lenses(),
    });
    monaco.languages.registerReferenceProvider("rust", {
        provideReferences: (m, pos) => state.references(pos.lineNumber, pos.column).map(({ range }) => ({ uri: m.uri, range })),
    });
    monaco.languages.registerDocumentHighlightProvider("rust", {
        provideDocumentHighlights: (_, pos) => state.references(pos.lineNumber, pos.column),
    });

    // class TokenState {
    //     constructor(line = 0) {
    //         this.line = line
    //     }
    //     clone() {
    //         let res = new TokenState(this.line)
    //         res.line += 1
    //         return res
    //     }

    //     equals(other) {
    //         return true
    //     }
    // }

    // monaco.languages.setTokensProvider("rust", {
    //     getInitialState: () => new TokenState(),
    //     tokenize(line, state) {

    //         const tokens = allTokens
    //             .filter(token => token.range.startLineNumber == state.line)
    //             .map(token => ({
    //                 startIndex: token.range.startColumn - 1,
    //                 scopes: token.tag,
    //             }))

    //         return {
    //             tokens,
    //             endState: new TokenState(state.line + 1)
    //         }
    //     }
    // })
})

monaco.editor.create(document.body, {
    theme: "vs-dark",
    value: example_code,
    language: "rust",
});
