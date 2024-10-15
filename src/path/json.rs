// use regex::Regex;
// use serde_json::Value;

// use super::JsonLike;

// /// compare sizes of json elements
// /// The method expects to get a number on the right side and array or string or object on the left
// /// where the number of characters, elements or fields will be compared respectively.
// pub fn size<T>(left: Vec<&T>, right: Vec<&T>) -> bool
// where
//     T: JsonLike,
// {
//     if let Some(Value::Number(n)) = right.first() {
//         if let Some(sz) = n.as_f64() {
//             for el in left.iter() {
//                 match el {
//                     Value::String(v) if v.len() == sz as usize => true,
//                     Value::Array(elems) if elems.len() == sz as usize => true,
//                     Value::Object(fields) if fields.len() == sz as usize => true,
//                     _ => return false,
//                 };
//             }
//             return true;
//         }
//     }
//     false
// }

// /// ensure the array on the left side is a subset of the array on the right side.
// //todo change the naive impl to sets
// pub fn sub_set_of<T>(left: Vec<&T>, right: Vec<&T>) -> bool {
//     if left.is_empty() {
//         return true;
//     }
//     if right.is_empty() {
//         return false;
//     }

//     if let Some(elems) = left.first().and_then(|e| e.as_array()) {
//         if let Some(Value::Array(right_elems)) = right.first() {
//             if right_elems.is_empty() {
//                 return false;
//             }

//             for el in elems {
//                 let mut res = false;

//                 for r in right_elems.iter() {
//                     if el.eq(r) {
//                         res = true
//                     }
//                 }
//                 if !res {
//                     return false;
//                 }
//             }
//             return true;
//         }
//     }
//     false
// }

// /// ensure at least one element in the array  on the left side belongs to the array on the right side.
// //todo change the naive impl to sets
// pub fn any_of<T>(left: Vec<&T>, right: Vec<&T>) -> bool {
//     if left.is_empty() {
//         return true;
//     }
//     if right.is_empty() {
//         return false;
//     }

//     if let Some(Value::Array(elems)) = right.first() {
//         if elems.is_empty() {
//             return false;
//         }

//         for el in left.iter() {
//             if let Some(left_elems) = el.as_array() {
//                 for l in left_elems.iter() {
//                     for r in elems.iter() {
//                         if l.eq(r) {
//                             return true;
//                         }
//                     }
//                 }
//             } else {
//                 for r in elems.iter() {
//                     if el.eq(&r) {
//                         return true;
//                     }
//                 }
//             }
//         }
//     }

//     false
// }

// /// ensure that the element on the left sides mathes the regex on the right side
// pub fn regex<T>(left: Vec<&T>, right: Vec<&T>) -> bool {
//     if left.is_empty() || right.is_empty() {
//         return false;
//     }

//     match right.first() {
//         Some(Value::String(str)) => {
//             if let Ok(regex) = Regex::new(str) {
//                 for el in left.iter() {
//                     if let Some(v) = el.as_str() {
//                         if regex.is_match(v) {
//                             return true;
//                         }
//                     }
//                 }
//             }
//             false
//         }
//         _ => false,
//     }
// }

// /// ensure that the element on the left side belongs to the array on the right side.
// pub fn inside<T>(left: Vec<&T>, right: Vec<&T>) -> bool {
//     if left.is_empty() {
//         return false;
//     }

//     match right.first() {
//         Some(Value::Array(elems)) => {
//             for el in left.iter() {
//                 if elems.contains(el) {
//                     return true;
//                 }
//             }
//             false
//         }
//         Some(Value::Object(elems)) => {
//             for el in left.iter() {
//                 for r in elems.values() {
//                     if el.eq(&r) {
//                         return true;
//                     }
//                 }
//             }
//             false
//         }
//         _ => false,
//     }
// }

// /// ensure the number on the left side is less the number on the right side
// pub fn less<T>(left: Vec<&T>, right: Vec<&T>) -> bool {
//     if left.len() == 1 && right.len() == 1 {
//         match (left.first(), right.first()) {
//             (Some(Value::Number(l)), Some(Value::Number(r))) => l
//                 .as_f64()
//                 .and_then(|v1| r.as_f64().map(|v2| v1 < v2))
//                 .unwrap_or(false),
//             _ => false,
//         }
//     } else {
//         false
//     }
// }

// /// compare elements
// pub fn eq<T>(left: Vec<&T>, right: Vec<&T>) -> bool {
//     if left.len() != right.len() {
//         false
//     } else {
//         left.iter().zip(right).map(|(a, b)| a.eq(&b)).all(|a| a)
//     }
// }


