use crate::service::UserService;

pub fn route_user_lookup(service: &UserService, id: u64) -> String {
    service
        .find_user(id)
        .map(|user| user.email)
        .unwrap_or_else(|| "missing".to_string())
}
