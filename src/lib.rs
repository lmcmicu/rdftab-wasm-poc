mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct JSON {
    content: String,
}

impl fmt::Display for JSON {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.content)?;
        write!(f, "\n")?;

        Ok(())
    }
}

#[wasm_bindgen]
impl JSON {
    pub fn new(content: &str) -> JSON {
        let content = content.to_string();
        JSON {
            content
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn thin_to_thick(&mut self) {
        self.content = self.content.replace("plugh", "flugh").to_string();
    }
}
