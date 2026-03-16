use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginated_response_serde() {
        let resp = PaginatedResponse {
            data: vec!["item1".to_string(), "item2".to_string()],
            meta: PaginationMeta {
                page: 1,
                per_page: 20,
                total: 42,
                total_pages: 3,
            },
        };

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: PaginatedResponse<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.data.len(), 2);
        assert_eq!(deserialized.meta.total, 42);
        assert_eq!(deserialized.meta.total_pages, 3);
        assert!(json.contains("\"perPage\""));
        assert!(json.contains("\"totalPages\""));
    }
}
