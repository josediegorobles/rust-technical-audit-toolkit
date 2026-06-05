pub mod api;
pub mod domain;
pub mod repository;
pub mod service;

pub fn health_status() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::health_status;

    #[test]
    fn reports_health() {
        assert_eq!(health_status(), "ok");
    }
}
