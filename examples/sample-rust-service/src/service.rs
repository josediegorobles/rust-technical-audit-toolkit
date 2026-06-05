use crate::{domain::User, repository::UserRepository};

#[derive(Debug)]
pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub fn find_user(&self, id: u64) -> Option<User> {
        self.repository.find(id)
    }
}
