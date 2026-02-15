use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};

use crate::schema::error::SchemaError;

type RegexCache = HashMap<String, Arc<regex::Regex>>;
type RegexCacheLock = RwLock<RegexCache>;

static REGEX_CACHE: OnceLock<RegexCacheLock> = OnceLock::new();

#[inline]
pub(crate) fn get_cached_regex(
    pattern: &str,
) -> Result<Arc<regex::Regex>, SchemaError> {
    let cache = REGEX_CACHE.get_or_init(|| RwLock::new(RegexCache::new()));

    if let Some(found) = cache
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(pattern)
    {
        return Ok(Arc::clone(found));
    }

    let compiled = Arc::new(regex::Regex::new(pattern).map_err(|e| {
        SchemaError::InvalidRegex(format!("Invalid pattern {pattern}: {e}"))
    })?);

    let mut lock =
        cache.write().unwrap_or_else(std::sync::PoisonError::into_inner);

    Ok(Arc::clone(lock.entry(pattern.to_owned()).or_insert(compiled)))
}
