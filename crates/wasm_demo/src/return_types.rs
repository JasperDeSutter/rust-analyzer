#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Hover {
    pub range: Range,
    pub contents: Vec<MarkdownString>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Range {
    pub startLineNumber: u32,
    pub startColumn: u32,
    pub endLineNumber: u32,
    pub endColumn: u32,
}

#[derive(Serialize)]
pub struct MarkdownString {
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct CodeLensSymbol {
    pub range: Range,
    pub command: Option<Command>,
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    // pub arguments: Vec<String>,
}
