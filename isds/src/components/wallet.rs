use super::*;
use blockchain_types::{coins_from, Address, BlockContents, BlockHeader, Transaction};
use nakamoto_consensus::NakamotoNodeState;
use std::collections::{BTreeSet, VecDeque};

pub struct Wallet {
    sim: SharedSimulation,
    cache: TransactionsCache,
    _context_handle: yew::context::ContextHandle<IsdsContext>,
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Rendered(RealSeconds),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub full_node: Option<Entity>,
    pub address: Address,
}

impl Component for Wallet {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context_data, _context_handle) = get_isds_context!(ctx, Self);

        let sim = context_data.sim;

        let mut cache = TransactionsCache::new(ctx.props().address.clone());
        cache.full_node = ctx.props().full_node;
        cache.update(&sim.borrow());

        Self {
            sim,
            cache,
            _context_handle,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <div class="box">
                { self.view_config_description() }
                { self.view_balance() }
                { self.view_transactions() }
                { self.view_buttons() }
            </div>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(_) => self.cache.update(&self.sim.borrow()),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.cache.reset();
        self.cache.full_node = ctx.props().full_node;
        self.cache.update(&self.sim.borrow())
    }
}

impl Wallet {
    fn view_config_description(&self) -> Html {
        html! {
            <div>
                { "Wallet of" }
                <span class="mx-2 is-size-5 is-family-code is-underlined">
                    { &self.cache.monitored_address }
                </span>
                { "connected to node" }
                <span class="ml-2 is-family-code">
                    { self.cache.full_node.map_or("None".to_string(), |id| self.sim.borrow().name(id)) }
                </span>
            </div>
        }
    }
    fn view_balance(&self) -> Html {
        html! {
            <div>
                <span class="is-size-4">
                    { format!("{} coins", coins_from(self.cache.total_value_confirmed())) }
                </span>
                if !self.cache.txes_unconfirmed.is_empty() {
                    <span class="ml-2 has-text-grey-light">
                        { format!("({:+} unconfirmed)", coins_from(self.cache.total_value_unconfirmed())) }
                    </span>
                }
            </div>
        }
    }
    fn view_transactions(&self) -> Html {
        let relevant_transactions = self.cache.iter_all_transactions_newest_first();
        let visible_transactions = relevant_transactions.clone().take(5);
        let value_before = if relevant_transactions.clone().count() > 5 {
            Some(
                relevant_transactions
                    .skip(5)
                    .map(|(_, tx)| self.cache.value_of(tx))
                    .sum(),
            )
        } else {
            None
        };
        html! {
            <table class="table is-fullwidth">
                <tbody>
                    {
                        visible_transactions.map(|(confirmations, tx)| {
                            let coin_value = coins_from(self.cache.value_of(tx));
                            let value_color_class = if confirmations < 1 {
                                "has-text-grey-light"
                            } else if coin_value < 0. {
                                "has-text-danger"
                            } else {
                                "has-text-success"
                            };
                            let icon_color_class = if confirmations < 3 {
                                "has-text-warning"
                            } else if coin_value < 0. {
                                "has-text-danger"
                            } else {
                                "has-text-success"
                            };
                            html! {
                                <tr>
                                    <td>
                                        <span class={classes!("icon", "is-size-6", icon_color_class)}>
                                            if confirmations < 3 {
                                                { format!("{}/3", confirmations) }
                                            } else {
                                                <i class="fas fa-circle"></i>
                                            }
                                        </span>
                                    </td>
                                    <td>
                                        <span class={classes!("has-text-grey-light", "is-family-code")}>
                                            { &tx.from }
                                        </span>
                                    </td>
                                    <td>
                                        <span class={classes!(value_color_class, "has-text-weight-medium")}>
                                            { format!("{:+}", coins_from(self.cache.value_of(tx))) }
                                        </span>
                                    </td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                    if let Some(value) = value_before {
                        <tr>
                            <td colspan=2 class="has-text-centered">
                                <span class="has-text-grey-light is-family-code">
                                    { "..." }
                                </span>
                            </td>
                            <td>
                                <span class="has-text-grey-light">
                                    { coins_from(value) }
                                </span>
                            </td>
                        </tr>
                    }
                </tbody>
            </table>
        }
    }
    fn view_buttons(&self) -> Html {
        html! {
            <div class="buttons">
                <button class="button">
                    <span class="icon">
                        <i class="fas fa-paper-plane fa-rotate-90"></i>
                    </span>
                    <span>
                        { "Request coins" }
                    </span>
                </button>
                <button class="button">
                    <span>
                        { "Send coins" }
                    </span>
                    <span class="icon">
                        <i class="fas fa-paper-plane"></i>
                    </span>
                </button>
            </div>
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct TransactionsCache {
    full_node: Option<Entity>,
    monitored_address: Address,
    tip: Option<BlockHeader>,
    txes_confirmed: VecDeque<(usize, Transaction)>,
    txids_unconfirmed: BTreeSet<Entity>,
    txes_unconfirmed: VecDeque<Transaction>,
}
impl TransactionsCache {
    fn new(monitored_address: Address) -> Self {
        Self {
            monitored_address,
            ..Self::default()
        }
    }
    fn reset(&mut self) -> bool {
        let new_self = Self::new(self.monitored_address.clone());
        let changed_something = *self == new_self;
        *self = new_self;
        changed_something
    }
    fn tip_height(&self) -> usize {
        if let Some(tip_header) = self.tip {
            tip_header.height
        } else {
            0
        }
    }
    fn total_value_confirmed(&self) -> i64 {
        self.txes_confirmed
            .iter()
            .map(|(_, tx)| self.value_of(tx))
            .sum()
    }
    fn total_value_unconfirmed(&self) -> i64 {
        self.txes_unconfirmed
            .iter()
            .map(|tx| self.value_of(tx))
            .sum()
    }
    fn iter_all_transactions_newest_first(
        &self,
    ) -> impl Iterator<Item = (usize, &Transaction)> + Clone {
        self.txes_unconfirmed.iter().map(|tx| (0, tx)).chain(
            self.txes_confirmed
                .iter()
                .map(|(height, tx)| (self.tip_height() - height + 1, tx)),
        )
    }
    fn update(&mut self, sim: &Simulation) -> bool {
        if let Some(full_node) = self.full_node {
            if let Some(state) = get_state(full_node, sim) {
                self.update_confirmed(&state, sim) || self.update_unconfirmed(&state, sim)
            } else {
                self.reset()
            }
        } else {
            self.reset()
        }
    }
    fn update_confirmed(&mut self, state: &NakamotoNodeState, sim: &Simulation) -> bool {
        let new_tip = state
            .tip()
            .map(|block_id| state.block_header(block_id).unwrap());
        if self.tip == new_tip {
            false // nothing changed
        } else {
            if let Some(new_tip) = new_tip {
                if new_tip.id_prev == self.tip.map(|tip| tip.id) {
                    self.update_confirmed_by_one_block(sim, new_tip.id);
                } else {
                    self.rebuild_confirmed_from_tip(sim, new_tip.id);
                }
            } else {
                self.txes_confirmed.clear();
            }
            self.tip = new_tip;
            true
        }
    }
    fn update_unconfirmed(&mut self, state: &NakamotoNodeState, sim: &Simulation) -> bool {
        if self.txids_unconfirmed == *state.txes_unconfirmed() {
            false
        } else if state
            .txes_unconfirmed()
            .is_superset(&self.txids_unconfirmed)
        {
            let mut changed = false;
            for txid in state
                .txes_unconfirmed()
                .difference(&self.txids_unconfirmed.clone())
                .copied()
            {
                let tx = get_transaction_unchecked(txid, sim);
                if self.is_relevant(&tx) {
                    self.txids_unconfirmed.insert(txid);
                    self.txes_unconfirmed.push_front((*tx).clone());
                    changed = true;
                }
            }
            changed
        } else {
            self.txids_unconfirmed.clear();
            self.txes_unconfirmed.clear();
            for &txid in state.txes_unconfirmed().iter() {
                let tx = get_transaction_unchecked(txid, sim);
                if self.is_relevant(&tx) {
                    self.txids_unconfirmed.insert(txid);
                    self.txes_unconfirmed.push_front((*tx).clone());
                }
            }
            true
        }
    }
    fn rebuild_confirmed_from_tip(&mut self, sim: &Simulation, block_id: Entity) {
        self.txes_confirmed.clear();
        let mut next_block = Some(block_id);
        while let Some(block_id) = next_block {
            next_block = self.update_confirmed_by_one_block(sim, block_id)
        }
    }
    fn update_confirmed_by_one_block(
        &mut self,
        sim: &Simulation,
        block_id: Entity,
    ) -> Option<Entity> {
        let (block_header, block_contents) = get_block_unchecked(block_id, sim);
        let block_height = block_header.height;
        for tx in block_contents
            .iter()
            .map(|tx_id| get_transaction_unchecked(*tx_id, sim))
        {
            if self.is_relevant(&tx) {
                self.txes_confirmed
                    .push_front((block_height, (*tx).clone()));
            }
        }
        block_header.id_prev
    }
    fn is_relevant(&self, tx: &Transaction) -> bool {
        tx.from == self.monitored_address || tx.to == self.monitored_address
    }
    fn value_of(&self, tx: &Transaction) -> i64 {
        if tx.from == self.monitored_address {
            -(tx.amount as i64)
        } else if tx.to == self.monitored_address {
            tx.amount as i64
        } else {
            0
        }
    }
}

fn get_block_unchecked(tx_id: Entity, sim: &Simulation) -> (BlockHeader, hecs::Ref<BlockContents>) {
    (
        get_block_header_unchecked(tx_id, sim),
        get_block_contents_unchecked(tx_id, sim),
    )
}

fn get_block_header_unchecked(tx_id: Entity, sim: &Simulation) -> BlockHeader {
    *sim.world.get::<BlockHeader>(tx_id).unwrap()
}

fn get_block_contents_unchecked(tx_id: Entity, sim: &Simulation) -> hecs::Ref<BlockContents> {
    sim.world.get::<BlockContents>(tx_id).unwrap()
}

fn get_transaction_unchecked(tx_id: Entity, sim: &Simulation) -> hecs::Ref<Transaction> {
    sim.world.get::<Transaction>(tx_id).unwrap()
}

fn get_state(node_id: Entity, sim: &Simulation) -> Option<hecs::Ref<NakamotoNodeState>> {
    sim.world.get::<NakamotoNodeState>(node_id).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn cache_registers_early_transactions() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(
            nakamoto_consensus::NakamotoConsensus::default(),
        ));

        // init network
        sim.do_now(SpawnRandomNodes(10));
        sim.do_now(MakeDelaunayNetwork);
        sim.work_until(SimSeconds::from(0.001)); // to make sure that some nodes are there

        // make fake "genesis payments" so that wallet balances are not 0
        let miner_node = sim.pick_random_node().unwrap();
        sim.do_now(ForSpecific(
            miner_node,
            nakamoto_consensus::BuildAndBroadcastTransaction::new(
                "CoinBroker25",
                "Bob",
                blockchain_types::toshis_from(1.) as u64,
            ),
        ));
        sim.do_now(ForSpecific(
            miner_node,
            nakamoto_consensus::BuildAndBroadcastTransaction::new(
                "Bob",
                "Alice",
                blockchain_types::toshis_from(0.5) as u64,
            ),
        ));
        // bury them beneath a couple of blocks
        sim.do_now(MultipleTimes::new(
            ForSpecific(miner_node, nakamoto_consensus::MineBlock),
            2,
        ));
        sim.work_until(SimSeconds::from(0.5));
        sim.do_now(ForRandomNode(nakamoto_consensus::MineBlock));
        sim.work_until(SimSeconds::from(1.));

        let wallet_node = sim.pick_random_other_node(miner_node).unwrap();
        let mut cache = TransactionsCache::new("Bob".to_string());
        cache.full_node = Some(wallet_node);
        cache.update(&mut sim);

        assert_eq!(cache.tip_height(), 3, "Cache didn't get all blocks?");
        assert_eq!(
            cache.iter_all_transactions_newest_first().count(),
            2,
            "Cache didn't get all transactions?"
        );
        assert_eq!(
            cache.total_value_confirmed(),
            blockchain_types::toshis_from(0.5),
            "Cache didn't calculate wallet value correctly?"
        );
    }
}
