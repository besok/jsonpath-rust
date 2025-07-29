use crate::parser::errors::JsonPathError;
use crate::parser::model::{JpQuery, Segment, Selector};
use crate::parser::{parse_json_path, Parsed};
use crate::query::{QueryPath, Queried};
use crate::JsonPath;
use serde_json::Value;
use std::borrow::Cow;
use std::fmt::Debug;

/// A trait that abstracts JSON-like data structures for JSONPath queries
///
/// This trait provides the essential operations needed to traverse and query
/// hierarchical data structures in a JSONPath-compatible way. Implementors of
/// this trait can be used with the JSONPath query engine.
///
/// The trait requires several standard type conversions to be implemented to
/// ensure that query operations can properly handle various data types.
///
/// # Type Requirements
///
/// Implementing types must satisfy these trait bounds:
/// - `Default`: Provides a default value for the type
/// - `Clone`: Allows creation of copies of values
/// - `Debug`: Enables debug formatting
/// - `From<&str>`: Conversion from string slices
/// - `From<bool>`: Conversion from boolean values
/// - `From<i64>`: Conversion from 64-bit integers
/// - `From<f64>`: Conversion from 64-bit floating point values
/// - `From<Vec<Self>>`: Conversion from vectors of the same type
/// - `From<String>`: Conversion from owned strings
/// - `PartialEq`: Allows equality comparisons
///
/// # Examples
///
/// The trait is primarily implemented for `serde_json::Value` to enable
/// JSONPath queries on JSON data structures:
///
/// ```
/// use serde_json::json;
/// use jsonpath_rust::JsonPath;
///
/// let data = json!({
///     "store": {
///         "books": [
///             {"title": "Book 1", "price": 10},
///             {"title": "Book 2", "price": 15}
///         ]
///     }
/// });
///
/// // Access data using the Queryable trait
/// let books = data.query("$.store.books[*].title").expect("no errors");
/// ```
pub trait Queryable
where
    Self: Default
        + Clone
        + Debug
        + for<'a> From<&'a str>
        + From<bool>
        + From<i64>
        + From<f64>
        + From<Vec<Self>>
        + From<String>
        + PartialEq,
{
    /// Retrieves a reference to the value associated with the given key.
    /// It is the responsibility of the implementation to handle enclosing single and double quotes.
    /// The key will be normalized (quotes trimmed, whitespace handled, the escape symbols handled) before lookup.
    fn get(&self, key: &str) -> Option<&Self>;

    fn as_array(&self) -> Option<&Vec<Self>>;

    fn as_object(&self) -> Option<Vec<(&str, &Self)>>;

    fn as_str(&self) -> Option<&str>;

    fn as_i64(&self) -> Option<i64>;
    fn as_f64(&self) -> Option<f64>;
    fn as_bool(&self) -> Option<bool>;

    /// Returns a null value.
    fn null() -> Self;

    fn extension_custom(_name: &str, _args: Vec<Cow<Self>>) -> Self {
        Self::null()
    }

    /// Retrieves a reference to the element at the specified path.
    /// The path is specified as a string and can be obtained from the query.
    ///
    /// # Arguments
    /// * `path` -  A json path to the element specified as a string (root, field, index only).
    fn reference<T>(&self, _path: T) -> Option<&Self>
    where
        T: Into<QueryPath>,
    {
        None
    }

    /// Retrieves a mutable reference to the element at the specified path.
    ///
    /// # Arguments
    /// * `path` -  A json path to the element specified as a string (root, field, index only).
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    /// use jsonpath_rust::JsonPath;
    /// use jsonpath_rust::query::queryable::Queryable;
    /// let mut json = json!({
    ///             "a": {
    ///                 "b": {
    ///                     "c": 42
    ///                 }
    ///             }
    ///         });
    ///         if let Some(path) = json.query_only_path("$.a.b.c").unwrap().first() {
    ///             if let Some(v) = json.reference_mut("$.a.b.c") {
    ///                 *v = json!(43);
    ///             }
    ///
    ///             assert_eq!(
    ///                 json,
    ///                 json!({
    ///                     "a": {
    ///                         "b": {
    ///                             "c": 43
    ///                         }
    ///                     }
    ///                 })
    ///             );
    /// }
    //// ```
    fn reference_mut<T>(&mut self, _path: T) -> Option<&mut Self>
    where
        T: Into<QueryPath>,
    {
        None
    }

    /// Deletes all elements matching the given JSONPath
    /// 
    /// # Arguments
    /// * `path` - JSONPath string specifying elements to delete
    /// 
    /// # Returns
    /// * `Ok(usize)` - Number of elements deleted
    /// * `Err(JsonPathError)` - If the path is invalid or deletion fails
    /// 
    /// # Examples
    /// ```
    /// use serde_json::json;
    /// use jsonpath_rust::JsonPath;
    /// use crate::jsonpath_rust::query::queryable::Queryable;
    /// 
    /// let mut data = json!({
    ///     "users": [
    ///         {"name": "Alice", "age": 30},
    ///         {"name": "Bob", "age": 25},
    ///         {"name": "Charlie", "age": 35}
    ///     ]
    /// });
    /// 
    /// // Delete users older than 30
    /// let deleted = data.delete_by_path("$.users[?(@.age > 30)]").unwrap();
    /// assert_eq!(deleted, 1);
    /// ```
    fn delete_by_path(&mut self, _path: &str) -> Queried<usize> {
        Err(JsonPathError::InvalidJsonPath("Deletion not supported".to_string()))
    }
}

impl Queryable for Value {
    fn get(&self, key: &str) -> Option<&Self> {
        let key = if key.starts_with("'") && key.ends_with("'") {
            key.trim_matches(|c| c == '\'')
        } else if key.starts_with('"') && key.ends_with('"') {
            key.trim_matches(|c| c == '"')
        } else {
            key
        };
        self.get(key)
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        self.as_array()
    }

    fn as_object(&self) -> Option<Vec<(&str, &Self)>> {
        self.as_object()
            .map(|v| v.into_iter().map(|(k, v)| (k.as_str(), v)).collect())
    }

    fn as_str(&self) -> Option<&str> {
        self.as_str()
    }

    fn as_i64(&self) -> Option<i64> {
        self.as_i64()
    }

    fn as_f64(&self) -> Option<f64> {
        self.as_f64()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_bool()
    }

    fn null() -> Self {
        Value::Null
    }

    /// Custom extension function for JSONPath queries.
    ///
    /// This function allows for custom operations to be performed on JSON data
    /// based on the provided `name` and `args`.
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the custom function.
    /// * `args` - A vector of `Cow<Self>` that holds the arguments for the custom function.
    ///
    /// # Returns
    ///
    /// Returns a `Self` value which is the result of the custom function. If the function
    /// name is not recognized, it returns `Self::null()`.
    ///
    /// # Custom Functions
    ///
    /// * `"in"` - Checks if the first argument is in the array provided as the second argument.
    ///   Example: `$.elems[?in(@, $.list)]` - Returns elements from $.elems that are present in $.list
    ///
    /// * `"nin"` - Checks if the first argument is not in the array provided as the second argument.
    ///   Example: `$.elems[?nin(@, $.list)]` - Returns elements from $.elems that are not present in $.list
    ///
    /// * `"none_of"` - Checks if none of the elements in the first array are in the second array.
    ///   Example: `$.elems[?none_of(@, $.list)]` - Returns arrays from $.elems that have no elements in common with $.list
    ///
    /// * `"any_of"` - Checks if any of the elements in the first array are in the second array.
    ///   Example: `$.elems[?any_of(@, $.list)]` - Returns arrays from $.elems that have at least one element in common with $.list
    ///
    /// * `"subset_of"` - Checks if all elements in the first array are in the second array.
    ///   Example: `$.elems[?subset_of(@, $.list)]` - Returns arrays from $.elems where all elements are present in $.list
    fn extension_custom(name: &str, args: Vec<Cow<Self>>) -> Self {
        match name {
            "in" => match args.as_slice() {
                [lhs, rhs] => match rhs.as_array() {
                    Some(elements) => elements.iter().any(|item| item == lhs.as_ref()).into(),
                    None => Self::null(),
                },
                _ => Self::null(),
            },
            "nin" => match args.as_slice() {
                [lhs, rhs] => match rhs.as_array() {
                    Some(elements) => (!elements.iter().any(|item| item == lhs.as_ref())).into(),
                    None => Self::null(),
                },
                _ => Self::null(),
            },
            "none_of" => match args.as_slice() {
                [lhs, rhs] => match (lhs.as_array(), rhs.as_array()) {
                    (Some(lhs_arr), Some(rhs_arr)) => lhs_arr
                        .iter()
                        .all(|lhs| !rhs_arr.iter().any(|rhs| lhs == rhs))
                        .into(),
                    _ => Self::null(),
                },
                _ => Self::null(),
            },
            "any_of" => match args.as_slice() {
                [lhs, rhs] => match (lhs.as_array(), rhs.as_array()) {
                    (Some(lhs_arr), Some(rhs_arr)) => lhs_arr
                        .iter()
                        .any(|lhs| rhs_arr.iter().any(|rhs| lhs == rhs))
                        .into(),
                    _ => Self::null(),
                },
                _ => Self::null(),
            },
            "subset_of" => match args.as_slice() {
                [lhs, rhs] => match (lhs.as_array(), rhs.as_array()) {
                    (Some(lhs_arr), Some(rhs_arr)) => lhs_arr
                        .iter()
                        .all(|lhs| rhs_arr.iter().any(|rhs| lhs == rhs))
                        .into(),
                    _ => Self::null(),
                },
                _ => Self::null(),
            },
            _ => Self::null(),
        }
    }

    fn reference<T>(&self, path: T) -> Option<&Self>
    where
        T: Into<QueryPath>,
    {
        convert_js_path(&path.into())
            .ok()
            .and_then(|p| self.pointer(p.as_str()))
    }

    fn reference_mut<T>(&mut self, path: T) -> Option<&mut Self>
    where
        T: Into<QueryPath>,
    {
        convert_js_path(&path.into())
            .ok()
            .and_then(|p| self.pointer_mut(p.as_str()))
    }

    fn delete_by_path(&mut self, path: &str) -> Queried<usize> {
        let mut deletions = Vec::new();
        for query_path in &self.query_only_path(path)? {
            if let Some(deletion_info) = parse_deletion_path(query_path)? {
                deletions.push(deletion_info);
            }
        }
        
        // Sort deletions to handle array indices correctly (delete from end to start)
        deletions.sort_by(|a, b|
            b.path_depth().cmp(&a.path_depth()).then_with(|| {
                match (a, b) {
                    (
                        DeletionInfo::ArrayIndex { index: idx_a, .. },
                        DeletionInfo::ArrayIndex { index: idx_b, .. },
                    ) => idx_b.cmp(idx_a),
                    _ => std::cmp::Ordering::Equal,
                }
            })
        );
        
        // Perform deletions
        let deleted_count = deletions
            .iter()
            .try_fold(0, |c, d| execute_deletion(self, d).map(|deleted| if deleted { c + 1 } else { c }))?;
        
        Ok(deleted_count)
    }
}

#[derive(Debug, Clone)]
enum DeletionInfo {
    ObjectField {
        parent_path: String,
        field_name: String,
    },
    ArrayIndex {
        parent_path: String,
        index: usize,
    },
    Root,
}

impl DeletionInfo {
    fn path_depth(&self) -> usize {
        match self {
            DeletionInfo::Root => 0,
            DeletionInfo::ObjectField { parent_path, .. } |
            DeletionInfo::ArrayIndex { parent_path, .. } => {
                parent_path.matches('/').count()
            }
        }
    }
}

fn parse_deletion_path(query_path: &str) -> Result<Option<DeletionInfo>, JsonPathError> {
    if query_path == "$" {
        return Ok(Some(DeletionInfo::Root));
    }
    
    let JpQuery { segments } = parse_json_path(query_path)?;
    
    if segments.is_empty() {
        return Ok(None);
    }
    
    let mut parent_path = String::new();
    let mut segments_iter = segments.iter().peekable();
    
    while let Some(segment) = segments_iter.next() {
        if segments_iter.peek().is_some() {
            // Not the last segment, add to parent path
            match segment {
                Segment::Selector(Selector::Name(name)) => {
                    parent_path.push_str(&format!("/{}", name.trim_matches(|c| c == '\'')));
                }
                Segment::Selector(Selector::Index(index)) => {
                    parent_path.push_str(&format!("/{}", index));
                }
                e => {
                    return Err(JsonPathError::InvalidJsonPath(format!(
                        "Unsupported segment to be deleted: {:?}",
                        e
                    )));
                }
            }
        } else {
            match segment {
                Segment::Selector(Selector::Name(name)) => {
                    let field_name = name.trim_matches(|c| c == '\'').to_string();
                    return Ok(Some(DeletionInfo::ObjectField {
                        parent_path,
                        field_name,
                    }));
                }
                Segment::Selector(Selector::Index(index)) => {
                    return Ok(Some(DeletionInfo::ArrayIndex {
                        parent_path,
                        index: *index as usize,
                    }));
                }
                e => {
                    return Err(JsonPathError::InvalidJsonPath(format!(
                        "Unsupported segment to be deleted: {:?}",
                        e
                    )));
                }
            }
        }
    }
    
    Ok(None)
}

fn execute_deletion(value: &mut Value, deletion: &DeletionInfo) -> Queried<bool> {
    match deletion {
        DeletionInfo::Root => {
            *value = Value::Null;
            Ok(true)
        }
        DeletionInfo::ObjectField { parent_path, field_name } => {
            let parent = if parent_path.is_empty() {
                value
            } else {
                value.pointer_mut(parent_path).ok_or_else(|| {
                    JsonPathError::InvalidJsonPath("Parent path not found".to_string())
                })?
            };
            
            if let Some(obj) = parent.as_object_mut() {
                Ok(obj.remove(field_name).is_some())
            } else {
                Err(JsonPathError::InvalidJsonPath(
                    "Parent is not an object".to_string()
                ))
            }
        }
        DeletionInfo::ArrayIndex { parent_path, index } => {
            let parent = if parent_path.is_empty() {
                value
            } else {
                value.pointer_mut(parent_path).ok_or_else(|| {
                    JsonPathError::InvalidJsonPath("Parent path not found".to_string())
                })?
            };
            
            if let Some(arr) = parent.as_array_mut() {
                if *index < arr.len() {
                    arr.remove(*index);
                    Ok(true)
                } else {
                    Ok(false) // Index out of bounds
                }
            } else {
                Err(JsonPathError::InvalidJsonPath(
                    "Parent is not an array".to_string()
                ))
            }
        }
    }
}

fn convert_js_path(path: &str) -> Parsed<String> {
    let JpQuery { segments } = parse_json_path(path)?;

    let mut path = String::new();
    for segment in segments {
        match segment {
            Segment::Selector(Selector::Name(name)) => {
                path.push_str(&format!("/{}", name.trim_matches(|c| c == '\'')));
            }
            Segment::Selector(Selector::Index(index)) => {
                path.push_str(&format!("/{}", index));
            }
            s => {
                return Err(JsonPathError::InvalidJsonPath(format!(
                    "Invalid segment: {:?}",
                    s
                )));
            }
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use crate::parser::Parsed;
    use crate::query::queryable::{convert_js_path, Queryable};
    use crate::query::Queried;
    use crate::JsonPath;
    use serde_json::{json, Value};

    #[test]
    fn in_smoke() -> Queried<()> {
        let json = json!({
            "elems": ["test", "t1", "t2"],
            "list": ["test", "test2", "test3"],
        });

        let res = json.query("$.elems[?in(@, $.list)]")?;

        assert_eq!(res, [&json!("test")]);

        Ok(())
    }
    #[test]
    fn nin_smoke() -> Queried<()> {
        let json = json!({
            "elems": ["test", "t1", "t2"],
            "list": ["test", "test2", "test3"],
        });

        let res = json.query("$.elems[?nin(@, $.list)]")?;

        assert_eq!(res, [&json!("t1"), &json!("t2")]);

        Ok(())
    }
    #[test]
    fn none_of_smoke() -> Queried<()> {
        let json = json!({
            "elems": [  ["t1", "_"], ["t2", "t5"], ["t4"]],
            "list": ["t1","t2", "t3"],
        });

        let res = json.query("$.elems[?none_of(@, $.list)]")?;

        assert_eq!(res, [&json!(["t4"])]);

        Ok(())
    }
    #[test]
    fn any_of_smoke() -> Queried<()> {
        let json = json!({
            "elems": [  ["t1", "_"], ["t4", "t5"], ["t4"]],
            "list": ["t1","t2", "t3"],
        });

        let res = json.query("$.elems[?any_of(@, $.list)]")?;

        assert_eq!(res, [&json!(["t1", "_"])]);

        Ok(())
    }
    #[test]
    fn subset_of_smoke() -> Queried<()> {
        let json = json!({
            "elems": [  ["t1", "t2"], ["t4", "t5"], ["t6"]],
            "list": ["t1","t2", "t3"],
        });

        let res = json.query("$.elems[?subset_of(@, $.list)]")?;

        assert_eq!(res, [&json!(["t1", "t2"])]);

        Ok(())
    }

    #[test]
    fn convert_paths() -> Parsed<()> {
        let r = convert_js_path("$.a.b[2]")?;
        assert_eq!(r, "/a/b/2");

        Ok(())
    }

    #[test]
    fn test_references() -> Parsed<()> {
        let mut json = json!({
            "a": {
                "b": {
                    "c": 42
                }
            }
        });

        let r = convert_js_path("$.a.b.c")?;

        if let Some(v) = json.pointer_mut(r.as_str()) {
            *v = json!(43);
        }

        assert_eq!(
            json,
            json!({
                "a": {
                    "b": {
                        "c": 43
                    }
                }
            })
        );

        Ok(())
    }
    #[test]
    fn test_js_reference() -> Parsed<()> {
        let mut json = json!({
            "a": {
                "b": {
                    "c": 42
                }
            }
        });

        if let Some(path) = json.query_only_path("$.a.b.c")?.first() {
            if let Some(v) = json.reference_mut(path) {
                *v = json!(43);
            }

            assert_eq!(
                json,
                json!({
                    "a": {
                        "b": {
                            "c": 43
                        }
                    }
                })
            );
        } else {
            panic!("no path found");
        }

        Ok(())
    }
    #[test]
    fn test_delete_object_field() {
        let mut data = json!({
            "users": {
                "alice": {"age": 30},
                "bob": {"age": 25}
            }
        });
        
        let deleted = data.delete_by_path("$.users.alice").unwrap();
        assert_eq!(deleted, 1);
        
        let expected = json!({
            "users": {
                "bob": {"age": 25}
            }
        });
        assert_eq!(data, expected);
    }
    
    #[test]
    fn test_delete_array_element() {
        let mut data = json!({
            "numbers": [1, 2, 3, 4, 5]
        });
        
        let deleted = data.delete_by_path("$.numbers[2]").unwrap();
        assert_eq!(deleted, 1);
        
        let expected = json!({
            "numbers": [1, 2, 4, 5]
        });
        assert_eq!(data, expected);
    }
    
    #[test]
    fn test_delete_multiple_elements() {
        let mut data = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25},
                {"name": "Charlie", "age": 35},
                {"name": "David", "age": 22}
            ]
        });
        
        // Delete users older than 24
        let deleted = data.delete_by_path("$.users[?(@.age > 24)]").unwrap();
        assert_eq!(deleted, 3);
        
        let expected = json!({
            "users": [
                {"name": "David", "age": 22}
            ]
        });
        assert_eq!(data, expected);
    }
    
    #[test]
    fn test_delete_nested_fields() {
        let mut data = json!({
            "company": {
                "departments": {
                    "engineering": {"budget": 100000},
                    "marketing": {"budget": 50000},
                    "hr": {"budget": 30000}
                }
            }
        });
        
        let deleted = data.delete_by_path("$.company.departments.marketing").unwrap();
        assert_eq!(deleted, 1);
        
        let expected = json!({
            "company": {
                "departments": {
                    "engineering": {"budget": 100000},
                    "hr": {"budget": 30000}
                }
            }
        });
        assert_eq!(data, expected);
    }
    
    #[test]
    fn test_delete_nonexistent_path() {
        let mut data = json!({
            "test": "value"
        });
        
        let deleted = data.delete_by_path("$.nonexistent").unwrap();
        assert_eq!(deleted, 0);
        
        // Data should remain unchanged
        let expected = json!({
            "test": "value"
        });
        assert_eq!(data, expected);
    }
    
    #[test]
    fn test_delete_root() {
        let mut data = json!({
            "test": "value"
        });
        
        let deleted = data.delete_by_path("$").unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(data, Value::Null);
    }
}