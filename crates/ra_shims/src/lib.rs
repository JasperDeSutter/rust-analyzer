#[cfg(not(target_arch = "wasm32"))]
pub type Instant = std::time::Instant;

#[cfg(target_arch = "wasm32")]
mod wasm_instant;
#[cfg(target_arch = "wasm32")]
pub use wasm_instant::Instant;
