#![cfg(target_arch = "wasm32")]

use ra_ide_api::{Analysis, FileId, FilePosition, LineCol};
use ra_syntax::TextRange;
use wasm_bindgen::prelude::*;

mod return_types;
use return_types::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace).expect("could not install logging hook");
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
    pub fn new(code: String) -> Self {
        let (analysis, file_id) = Analysis::from_single_file(code);

        Self { analysis, file_id }
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
        log::trace!("on_dot_typed");
        let res = self.analysis.on_dot_typed(pos).unwrap();

        log::debug!("{:?}", res);
    }

    pub fn hover(&self, line_number: u32, column: u32) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::trace!("hover");
        let info = match self.analysis.hover(pos).unwrap() {
            Some(info) => info,
            _ => return JsValue::NULL,
        };

        let value = info.info.to_markup();
        let hover =
            Hover { contents: vec![MarkdownString { value }], range: self.range(info.range) };

        serde_wasm_bindgen::to_value(&hover).unwrap()
    }
}
