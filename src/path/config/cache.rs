use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use regex::{Error, Regex};
use serde_json::Value;

#[derive(Clone)]
pub enum RegexCache<T = DefaultRegexCacheInst>
    where T: Clone + RegexCacheInst
{
    Absent,
    Implemented(T),
}

impl<T> RegexCache<T>
    where T: Clone + RegexCacheInst
{
    pub fn is_implemented(&self) -> bool {
        match self {
            RegexCache::Absent => false,
            RegexCache::Implemented(_) => true
        }
    }
    pub fn get_instance(&self) -> Result<&T, RegexCacheError> {
        match self {
            RegexCache::Absent => Err(RegexCacheError::new("the instance is absent".to_owned())),
            RegexCache::Implemented(inst) => Ok(inst)
        }
    }

    pub fn instance(instance: T) -> Self {
        RegexCache::Implemented(instance)
    }
}

impl Default for RegexCache {
    fn default() -> Self {
        RegexCache::Absent
    }
}

pub trait RegexCacheInst {
    fn validate(&self, regex: &str, values: Vec<&Value>) -> Result<bool, RegexCacheError>;
}


#[derive(Default, Debug, Clone)]
pub struct DefaultRegexCacheInst {
    cache: Arc<Mutex<HashMap<String, Regex>>>,
}

impl RegexCacheInst for DefaultRegexCacheInst {
    fn validate(&self, regex: &str, values: Vec<&Value>) -> Result<bool, RegexCacheError> {
        let mut cache = self.cache.lock()?;
        if cache.contains_key(regex) {
            let r = cache.get(regex).unwrap();
             Ok(validate(r, values))
        } else {
            let new_reg = Regex::new(regex)?;
            let result = validate(&new_reg, values);
            cache.insert(regex.to_owned(), new_reg);
            Ok(result)
        }
    }
}

fn validate(r: &Regex, values: Vec<&Value>) -> bool {
    for el in values.iter() {
        if let Some(v) = el.as_str() {
            if r.is_match(v) {
                return true;
            }
        }
    }
    false
}

pub struct RegexCacheError {
    pub reason: String,
}

impl From<Error> for RegexCacheError {
    fn from(value: Error) -> Self {
        RegexCacheError::new(value.to_string())
    }
}

impl<T> From<PoisonError<T>> for RegexCacheError {
    fn from(value: PoisonError<T>) -> Self {
        RegexCacheError::new(value.to_string())
    }
}

impl RegexCacheError {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}
