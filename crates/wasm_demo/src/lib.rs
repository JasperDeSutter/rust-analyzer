use ra_ide_api::Analysis;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Trace).expect("could not install logging hook");

    let (_analysis, _file_id) = Analysis::from_single_file(
        r#"
    fn main() {
        println!("Hello, world!");
    }
"#
        .to_owned(),
    );
}
