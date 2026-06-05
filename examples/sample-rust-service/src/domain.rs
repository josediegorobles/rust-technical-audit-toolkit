#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub email: String,
}

impl User {
    pub fn new(id: u64, email: impl Into<String>) -> Self {
        Self {
            id,
            email: email.into(),
        }
    }
}
