use serde_json::Value;
use crate::path::structures::{JsonPath, JsonPathIndex};
use crate::path::path::{Path, EmptyPath, RootPointer, ObjectField};

mod path;
mod structures;

#[cfg(test)]
mod tests {
    use crate::path::structures::{JsonPath, JsonPathIndex, parse};

    #[test]
    fn test() {}
}
