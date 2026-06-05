use crate::domain::User;

#[derive(Debug, Default)]
pub struct UserRepository {
    users: Vec<User>,
}

impl UserRepository {
    pub fn with_seed_data() -> Self {
        Self {
            users: vec![User::new(1, "cto@example.com")],
        }
    }

    pub fn find(&self, id: u64) -> Option<User> {
        self.users.iter().find(|user| user.id == id).cloned()
    }
}
