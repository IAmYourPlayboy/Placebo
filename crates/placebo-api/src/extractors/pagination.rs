use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    50
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) as i64) * (self.per_page as i64)
    }

    pub fn limit(&self) -> i64 {
        self.per_page.min(200) as i64
    }
}
