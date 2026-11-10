use regex::Regex;
use std::borrow::Cow;

/// What an evaluation can reuse from one node, document or query to the next.
///
/// The library keeps no state of its own. Whatever is worth reusing lives in a context that
/// the caller creates, passes to [`JpQuery::execute_with_context`] and drops when it is no
/// longer needed. The default context reuses nothing.
///
/// ```
/// use jsonpath_rust::parser::parse_json_path;
/// use jsonpath_rust::query::JsonPathEvalContext;
/// use serde_json::json;
///
/// let query = parse_json_path("$[?search(@.title, 'of')]")?;
/// let mut ctx = JsonPathEvalContext::with_regex_cache();
///
/// let first = json!([{"title": "Sayings of the Century"}, {"title": "Moby Dick"}]);
/// let second = json!([{"title": "Sword of Honour"}]);
///
/// // The pattern is compiled once for the three titles.
/// assert_eq!(query.execute_with_context(&first, &mut ctx)?.len(), 1);
/// assert_eq!(query.execute_with_context(&second, &mut ctx)?.len(), 1);
/// # Ok::<(), jsonpath_rust::parser::errors::JsonPathError>(())
/// ```
///
/// [`JpQuery::execute_with_context`]: crate::parser::model::JpQuery::execute_with_context
#[derive(Debug, Clone, Default)]
pub struct JsonPathEvalContext {
    regex_cache: Option<RegexCache>,
}

impl JsonPathEvalContext {
    /// A context that keeps the patterns `match()` and `search()` compile, so that a pattern
    /// is compiled once instead of once per node.
    ///
    /// It keeps the 8 patterns used last, whichever queries and documents they come from.
    pub fn with_regex_cache() -> Self {
        JsonPathEvalContext {
            regex_cache: Some(RegexCache::default()),
        }
    }

    /// Returns the compiled `pattern`, from the cache if the context has one.
    pub(crate) fn regex(&mut self, pattern: &str) -> Result<Cow<'_, Regex>, regex::Error> {
        match &mut self.regex_cache {
            Some(cache) => cache.get_or_compile(pattern).map(Cow::Borrowed),
            None => Regex::new(pattern).map(Cow::Owned),
        }
    }
}

/// The compiled patterns, the most recently used first.
#[derive(Debug, Clone, Default)]
struct RegexCache(Vec<Regex>);

impl RegexCache {
    /// A pattern can come from the document, so the cache must not grow with the data.
    const CAPACITY: usize = 8;

    fn get_or_compile(&mut self, pattern: &str) -> Result<&Regex, regex::Error> {
        match self.0.iter().position(|regex| regex.as_str() == pattern) {
            Some(hit) => self.0[..=hit].rotate_right(1),
            None => {
                let regex = Regex::new(pattern)?;
                self.0.truncate(Self::CAPACITY - 1);
                self.0.insert(0, regex);
            }
        }
        Ok(&self.0[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_json_path;
    use serde_json::json;

    fn patterns(cache: &RegexCache) -> Vec<&str> {
        cache.0.iter().map(Regex::as_str).collect()
    }

    #[test]
    fn cache_keeps_the_patterns_used_last() {
        let mut cache = RegexCache::default();
        for i in 0..RegexCache::CAPACITY {
            cache.get_or_compile(&format!("p{i}")).unwrap();
        }
        // p0 is used again, which leaves p1 as the one to give up for the next pattern
        cache.get_or_compile("p0").unwrap();
        cache.get_or_compile("next").unwrap();

        assert_eq!(
            patterns(&cache),
            ["next", "p0", "p7", "p6", "p5", "p4", "p3", "p2"]
        );
    }

    #[test]
    fn cache_skips_an_invalid_pattern() {
        let mut cache = RegexCache::default();
        cache.get_or_compile("a").unwrap();

        assert!(cache.get_or_compile("a[").is_err());
        assert_eq!(patterns(&cache), ["a"]);
    }

    #[test]
    fn default_context_keeps_nothing() {
        let mut ctx = JsonPathEvalContext::default();

        assert!(ctx.regex("a").is_ok());
        assert!(ctx.regex_cache.is_none());
    }

    // match() and search() have to get the context wherever they are in the query.
    #[test]
    fn context_reaches_every_function_call() {
        let json = json!({
            "a": [{"b": "x1", "c": {"b": "x1"}}, {"b": "x2", "c": {"b": "x2"}}],
            "p": "x."
        });

        for (path, compiled) in [
            ("$.a[?match(@.b, 'x.')]", 1),
            ("$.a[?!search(@.b, 'x.')]", 1),
            ("$.a[?match(@.b, $.p)]", 1),
            ("$.a[?@.b == 'y' || match(@.b, 'x.')]", 1),
            ("$.a[?@.b && (match(@.b, 'x.'))]", 1),
            ("$..[?match(@.b, 'x.')]", 1),
            ("$.a[?match(@.b, 'x1'), ?match(@.b, 'x2')]", 2),
            ("$.a[?@[?match(@.b, 'x.')]]", 1),
            ("$[?$.a[?match(@.b, 'x.')]]", 1),
            ("$.a[?count(@[?match(@.b, 'x.')]) == 1]", 1),
            ("$.a[?1 == count(@[?match(@.b, 'x.')])]", 1),
            ("$.a[?value(@[?match(@.b, 'x.')]) == 'x1']", 1),
            ("$.a[?length(@[?match(@.b, 'x.')]) == 1]", 1),
            ("$.a[?match(@[?match(@.b, 'x.')], 'y')]", 1),
            ("$.a[?match(@.b, @[?match(@.b, 'x.')])]", 1),
            ("$.a[?search(@[?search(@.b, 'x.')], 'y')]", 1),
            ("$.a[?search(@.b, @[?search(@.b, 'x.')])]", 1),
            ("$.a[?in(@.b, $.a[?match(@.b, 'x.')])]", 1),
            ("$.a[?in(!match(@.b, 'x.'), $.p)]", 1),
        ] {
            let mut ctx = JsonPathEvalContext::with_regex_cache();
            parse_json_path(path)
                .and_then(|query| query.execute_with_context(&json, &mut ctx))
                .expect(path);

            assert_eq!(ctx.regex_cache.unwrap().0.len(), compiled, "{path}");
        }
    }
}
