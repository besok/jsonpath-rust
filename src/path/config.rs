pub mod cache;

use crate::path::config::cache::{RegexCache};

#[derive(Clone, Default)]
pub struct JsonPathConfig {
    pub regex_cache: RegexCache,
}

impl JsonPathConfig {
    pub fn new(regex_cache: RegexCache) -> Self {
        Self { regex_cache }
    }
}

