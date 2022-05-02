#[derive(Debug, Clone, Eq, PartialEq)]
pub struct User {
    pub name: String,
    pub wallet_node: Option<isds::Entity>,
    pub show_wallet: bool,
}

impl User {
    pub fn new(name: &str, wallet_node: Option<isds::Entity>, show_wallet: bool) -> Self {
        Self {
            name: name.to_string(),
            wallet_node,
            show_wallet,
        }
    }
}
