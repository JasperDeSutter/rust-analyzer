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

const modeId = "ra-rust" // not "rust" to circumvent conflict
monaco.languages.register({ // language for editor
    id: modeId,
});
monaco.languages.register({ // language for hover info
    id: "rust",
});

monaco.languages.onLanguage(modeId, async () => {
    const { WorldState } = await wasm_demo;

    // let editActionId = myEditor.addCommand(0, (...args) => {
    //     console.warn(args)
    // })

    const state = new WorldState();

    const [model] = monaco.editor.getModels();
    let allTokens = []

    function update() {
        const res = state.update(model.getValue())
        monaco.editor.setModelMarkers(model, modeId, res.diagnostics)
        allTokens = res.highlights
    }
    update()

    let timeout = 0
    model.onDidChangeContent(e => {
        update()
        // if (timeout != 0) {
        //     clearTimeout(timeout)
        // }
        // console.warn('update')
        // timeout = setTimeout(update)
    })

    monaco.languages.setLanguageConfiguration(modeId, rust_conf.conf);
    monaco.languages.setLanguageConfiguration("rust", rust_conf.conf);
    monaco.languages.setMonarchTokensProvider("rust", rust_conf.language);

    // This panics when typing:
    // monaco.languages.registerCompletionItemProvider(modeId, {
    //     triggerCharacters: ["."],
    //     provideCompletionItems: (_, pos) => state.on_dot_typed(pos.lineNumber, pos.column),
    // });

    monaco.languages.registerHoverProvider(modeId, {
        provideHover: (_, pos) => state.hover(pos.lineNumber, pos.column),
    });
    monaco.languages.registerCodeLensProvider(modeId, {
        provideCodeLenses: () => state.code_lenses(),
    });
    monaco.languages.registerReferenceProvider(modeId, {
        provideReferences: (m, pos) => state.references(pos.lineNumber, pos.column).map(({ range }) => ({ uri: m.uri, range })),
    });
    monaco.languages.registerDocumentHighlightProvider(modeId, {
        provideDocumentHighlights: (_, pos) => state.references(pos.lineNumber, pos.column),
    });
    monaco.languages.registerCodeActionProvider(modeId, {
        provideCodeActions: (_, range) => (console.warn("provide actions"), state.actions(range.startLineNumber, range.startColumn, range.endLineNumber, range.endColumn)),
    })

    class TokenState {
        constructor(line = 0) {
            this.line = line
        }
        clone() {
            let res = new TokenState(this.line)
            res.line += 1
            return res
        }

        equals(other) {
            return true
        }
    }

    function fixTag(tag) {
        switch (tag) {
            case "literal": return "number";
            case "function": return "identifier";
            default: return tag;
        }
    }

    monaco.languages.setTokensProvider(modeId, {
        getInitialState: () => new TokenState(),
        tokenize(line, state) {
            const filteredTokens = allTokens
                .filter(token => token.range.startLineNumber == state.line)

            const tokens = filteredTokens.map(token => ({
                startIndex: token.range.startColumn - 1,
                scopes: fixTag(token.tag),
            }))
            // add tokens inbetween highlighted ones to remove color artifacts 
            tokens.push(...filteredTokens
                .filter((tok, i) => !tokens[i + 1] || tokens[i + 1].startIndex > (tok.range.endColumn - 1))
                .map(token => ({
                    startIndex: token.range.endColumn - 1,
                    scopes: 'operator',
                })))
            tokens.sort((a, b) => a.startIndex - b.startIndex)

            return {
                tokens,
                endState: new TokenState(state.line + 1)
            }
        }
    })
})

const myEditor = monaco.editor.create(document.body, {
    theme: "vs-dark",
    value: example_code,
    language: modeId,
});
