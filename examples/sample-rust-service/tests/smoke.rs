use sample_rust_service::{
    api::route_user_lookup, repository::UserRepository, service::UserService,
};

#[test]
fn resolves_seed_user() {
    let service = UserService::new(UserRepository::with_seed_data());

    assert_eq!(route_user_lookup(&service, 1), "cto@example.com");
}
