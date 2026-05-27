#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tool {
    pub id: i32,
    pub key: &'static str,
    pub enabled: bool,
}
