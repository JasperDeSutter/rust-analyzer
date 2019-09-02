#![cfg(target_arch = "wasm32")]
#![allow(non_snake_case)]

use ra_ide_api::{Analysis, FileId, FilePosition, FileRange, LineCol, Severity};
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
        let (analysis, file_id) = Analysis::from_single_file(code);
        self.analysis = analysis;
        self.file_id = file_id;

        let highlights: Vec<_> = self
            .analysis
            .highlight(file_id)
            .unwrap()
            .into_iter()
            .map(|hl| Highlight { tag: Some(hl.tag), range: self.range(hl.range) })
            .collect();

        let diagnostics: Vec<_> = self
            .analysis
            .diagnostics(self.file_id)
            .unwrap()
            .into_iter()
            .map(|d| {
                let Range { startLineNumber, startColumn, endLineNumber, endColumn } =
                    self.range(d.range);
                Diagnostic {
                    message: d.message,
                    severity: match d.severity {
                        Severity::Error => 8,       // monaco MarkerSeverity.Error
                        Severity::WeakWarning => 1, // monaco MarkerSeverity.Hint
                    },
                    startLineNumber,
                    startColumn,
                    endLineNumber,
                    endColumn,
                }
            })
            .collect();

        serde_wasm_bindgen::to_value(&UpdateResult { diagnostics, highlights }).unwrap()
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

    fn file_range(
        &self,
        start_line: u32,
        start_col_utf16: u32,
        end_line: u32,
        end_col_utf16: u32,
    ) -> FileRange {
        let from = self.file_pos(start_line, start_col_utf16);
        let to = self.file_pos(end_line, end_col_utf16);

        FileRange { file_id: self.file_id, range: TextRange::from_to(from.offset, to.offset) }
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
            info.into_iter().map(|r| Highlight { tag: None, range: self.range(r.range) }).collect();
        serde_wasm_bindgen::to_value(&res).unwrap()
    }

    pub fn actions(
        &self,
        start_line_number: u32,
        start_column: u32,
        end_line_number: u32,
        end_column: u32,
    ) -> JsValue {
        let range = self.file_range(start_line_number, start_column, end_line_number, end_column);
        log::warn!("actions");

        // pub id: AssistId,
        // pub change: SourceChange,
        let result: Vec<_> = self.analysis.assists(range).unwrap(); //.iter().map(|assist| ).collect()
        log::warn!("{:#?}", result);

        JsValue::NULL
        // serde_wasm_bindgen::to_value(&result).unwrap()
    }

    pub fn prepare_rename(&self, line_number: u32, column: u32) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("prepare_rename");
        let refs = match self.analysis.find_all_refs(pos).unwrap() {
            None => return JsValue::NULL,
            Some(refs) => refs,
        };

        let declaration = refs.declaration();
        let range = self.range(declaration.range());
        let text = declaration.name().to_string();

        serde_wasm_bindgen::to_value(&RenameLocation { range, text }).unwrap()
    }

    pub fn rename(&self, line_number: u32, column: u32, new_name: &str) -> JsValue {
        let pos = self.file_pos(line_number, column);
        log::warn!("rename");
        let change = match self.analysis.rename(pos, new_name) {
            Ok(Some(change)) => change,
            _ => return JsValue::NULL,
        };

        let result: Vec<_> = change
            .source_file_edits
            .iter()
            .flat_map(|sfe| sfe.edit.as_atoms())
            .map(|atom| TextEdit { range: self.range(atom.delete), text: atom.insert.clone() })
            .collect();

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}
