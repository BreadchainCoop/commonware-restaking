#[derive(Debug, Clone)]
pub struct CreatorConfig {
    pub polling_interval_ms: u64,
    pub timeout_ms: u64,
}

impl Default for CreatorConfig {
    fn default() -> Self {
        Self {
            polling_interval_ms: 100,
            timeout_ms: 5000,
        }
    }
}
