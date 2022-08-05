use rand::{seq::SliceRandom, thread_rng, Rng};

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

pub fn random_transaction(sim: &mut isds::Simulation, origin_node: isds::Entity) {
    sim.do_now(isds::ForSpecific(origin_node, random_transaction_command()));
}

pub fn random_transaction_from_random_node(sim: &mut isds::Simulation) {
    sim.do_now(isds::ForRandomNode(random_transaction_command()));
}

fn random_transaction_command() -> isds::nakamoto_consensus::BuildAndBroadcastTransaction {
    let mut rng = thread_rng();
    let addresses = "CDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    isds::nakamoto_consensus::BuildAndBroadcastTransaction::from(
        addresses.choose(&mut rng).unwrap(),
        addresses.choose(&mut rng).unwrap(),
        isds::blockchain_types::toshis_from(rng.gen_range(1..100) as f64) as u64,
    )
}
