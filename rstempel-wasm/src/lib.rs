use wasm_bindgen::prelude::*;
use rstempel::Stem;
use rstempel::rust::STEMMER;

#[wasm_bindgen]
pub fn stem(word: &str) -> String {
    STEMMER.stem(word).into_owned()
}

#[wasm_bindgen]
pub fn rstempel_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
