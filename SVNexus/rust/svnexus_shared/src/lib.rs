use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[tarpc::service]
pub trait TunnelService {
    async fn authenticate(username: Option<String>) -> User;
    async fn verify_server_hostkey(r#type: String, key: String) -> bool;
}
