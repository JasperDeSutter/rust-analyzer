#![cfg(target_arch = "wasm32")]
#![allow(non_snake_case)]

use ra_ide_api::{Analysis, FileId, FilePosition, LineCol, Severity};
use ra_syntax::{SyntaxKind, TextRange};
use wasm_bindgen::prelude::*;

mod return_types;
use return_types::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Warn).expect("could not install logging hook");
    log::info!("worker initialized")
}

#[wasm_bindgen]
pub struct WorldState {
    analysis: Analysis,
    file_id: FileId,
}

#[wasm_bindgen]
impl WorldState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let (analysis, file_id) = Analysis::from_single_file("".to_owned());
        Self { analysis, file_id }
    }

    pub fn update(&mut self, code: String) -> JsValue {
        // TODO: how to update analyisis source?
        let (analysis, file_id) = Analysis::from_single_file(code);
        self.analysis = analysis;
        self.file_id = file_id;


        // let result: Vec<_> = self.analysis.highlight(file_id).unwrap().iter().map(|hl| Highlight {
        //     tag: hl.tag,
        //     range: self.range(hl.range)
        // }).collect();

        let diagnostics: Vec<_> = self
            .analysis
            .diagnostics(self.file_id)
            .unwrap()
            .iter()
            .map(|d| {
                let Range { startLineNumber, startColumn, endLineNumber, endColumn } =
                    self.range(d.range);
                Diagnostic {
                    message: d.message.clone(),
                    severity: match d.severity {
                        Severity::Error => 8,
                        Severity::WeakWarning => 1,
                    },
                    startLineNumber,
                    startColumn,
                    endLineNumber,
                    endColumn,
                }
            })
            .collect();

        serde_wasm_bindgen::to_value(&UpdateResult { diagnostics }).unwrap()
    }

    fn file_pos(&self, line: u32, col_utf16: u32) -> FilePosition {
        // monaco doesn't work zero-based
        let line_col = LineCol { line: line - 1, col_utf16: col_utf16 - 1 };
        let offset = self.analysis.file_line_index(self.file_id).unwrap().offset(line_col);
        FilePosition { file_id: self.file_id, offset }
    }

    fn range(&self, text_range: TextRange) -> Range {
        let line_index = self.analysis.file_line_index(self.file_id).unwrap();
        let start = line_index.line_col(text_range.start());
        let end = line_index.line_col(text_range.end());

        Range {
            startLineNumber: start.line + 1,
            startColumn: start.col_utf16 + 1,
            endLineNumber: end.line + 1,
            endColumn: end.col_utf16 + 1,
        }
    }

    pub fn on_dot_typed(&self, line_number: u32, column: u32) {
        let pos = self.file_pos(line_number, column);
        log::warn!("on_dot_typed");
        let res = self.analysis.on_dot_typed(pos).unwrap();

        log::debug!("{:?}", res);
    }

    pub fn hover(&self, line_number: u32, column: u32) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("hover");
        let info = match self.analysis.hover(pos).unwrap() {
            Some(info) => info,
            _ => return JsValue::NULL,
        };

        let value = info.info.to_markup();
        let hover =
            Hover { contents: vec![MarkdownString { value }], range: self.range(info.range) };

        serde_wasm_bindgen::to_value(&hover).unwrap()
    }

    pub fn code_lenses(&self) -> JsValue {
        log::warn!("code_lenses");

        let results: Vec<_> = self
            .analysis
            .file_structure(self.file_id)
            .unwrap()
            .into_iter()
            .filter(|it| match it.kind {
                SyntaxKind::TRAIT_DEF | SyntaxKind::STRUCT_DEF | SyntaxKind::ENUM_DEF => true,
                _ => false,
            })
            .map(|it| {
                let range = self.range(it.node_range);
                CodeLensSymbol {
                    range,
                    command: Some(Command {
                        id: "rust-analyzer.showReferences".into(),
                        title: "0 implementations".into(), // TODO: actual implementation, or use resolve
                    }),
                }
            })
            .collect();

        serde_wasm_bindgen::to_value(&results).unwrap()
    }

    pub fn references(&self, line_number: u32, column: u32) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("references");
        let info = match self.analysis.find_all_refs(pos).unwrap() {
            Some(info) => info,
            _ => return JsValue::NULL,
        };

        let res: Vec<_> =
            info.into_iter().map(|r| Highlight { range: self.range(r.range) }).collect();
        serde_wasm_bindgen::to_value(&res).unwrap()
    }
}
