#![allow(non_snake_case)]
use serde::Serialize;

#[derive(Serialize)]
pub struct Hover {
    pub range: Range,
    pub contents: Vec<MarkdownString>,
}

#[derive(Serialize)]
#[derive(Clone, Copy)]
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
