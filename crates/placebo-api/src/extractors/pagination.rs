use serde::{Deserialize, Deserializer};

/// Pagination query params shared across list endpoints.
///
/// Numbers are parsed via a string-tolerant deserializer because axum's
/// default `Query` extractor uses `serde_urlencoded`, which does not
/// coerce string query values into numeric Rust types. Without this,
/// `?per_page=50` would 400 with "invalid type: string "50", expected u32".
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page", deserialize_with = "deserialize_u32_lenient")]
    pub page: u32,
    #[serde(default = "default_per_page", deserialize_with = "deserialize_u32_lenient")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    50
}

/// Accept either a JSON-style integer or a urlencoded string-encoded integer
/// ("50"). Returning a clear error keeps the 400 message useful.
fn deserialize_u32_lenient<'de, D: Deserializer<'de>>(d: D) -> Result<u32, D::Error> {
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumOrStr {
        Num(u32),
        Str(String),
    }

    match NumOrStr::deserialize(d)? {
        NumOrStr::Num(n) => Ok(n),
        NumOrStr::Str(s) => s.parse::<u32>().map_err(|e| {
            D::Error::custom(format!("expected non-negative integer, got {s:?}: {e}"))
        }),
    }
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) as i64) * (self.per_page as i64)
    }

    pub fn limit(&self) -> i64 {
        self.per_page.min(200) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::value::{Error as ValueError, MapDeserializer};
    use serde::de::IntoDeserializer;

    fn parse(pairs: &[(&str, &str)]) -> Result<PaginationParams, ValueError> {
        let map = pairs.iter().copied().collect::<std::collections::HashMap<_, _>>();
        let de = MapDeserializer::new(map.into_iter().map(|(k, v)| (k, v.into_deserializer())));
        PaginationParams::deserialize(de)
    }

    #[test]
    fn parses_string_numbers_from_query_string() {
        let p = parse(&[("page", "2"), ("per_page", "25")]).unwrap();
        assert_eq!(p.page, 2);
        assert_eq!(p.per_page, 25);
    }

    #[test]
    fn defaults_apply_when_param_missing() {
        let p = parse(&[]).unwrap();
        assert_eq!(p.page, default_page());
        assert_eq!(p.per_page, default_per_page());
    }

    #[test]
    fn rejects_non_numeric_strings() {
        let err = parse(&[("page", "abc")]).unwrap_err();
        assert!(err.to_string().contains("abc"), "err = {err}");
    }

    #[test]
    fn limit_caps_at_200() {
        let p = parse(&[("per_page", "500")]).unwrap();
        assert_eq!(p.limit(), 200);
    }
}
