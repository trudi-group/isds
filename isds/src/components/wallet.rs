use super::*;
use blockchain_types::{Address, BlockContents, BlockHeader, Transaction};
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
    // TODO: wallet addresses to filter by
}

impl Component for Wallet {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (context_data, _context_handle) = get_isds_context!(ctx, Self);

        let sim = context_data.sim;

        let mut cache = TransactionsCache::new();
        cache.full_node = ctx.props().full_node;
        cache.update(&sim.borrow());

        Self {
            sim,
            cache,
            _context_handle,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
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
            <div class="box">
                <div>
                    { "Wallet of" }
                    <span class="mx-2 is-size-5 is-family-code is-underlined">
                        { "Bob" }
                    </span>
                    { "connected to node" }
                    <span class="ml-2 is-family-code">
                        { format!("{}", self.cache.full_node.map_or("None".to_string(), |id| self.sim.borrow().name(id))) }
                    </span>
                </div>
                <div>
                    <span class="is-size-4">
                        { format!("{} coins", to_coin(self.cache.total_value_confirmed())) }
                    </span>
                    <span class="ml-2">
                        { format!("({:+} unconfirmed)", to_coin(self.cache.total_value_unconfirmed())) }
                    </span>
                </div>
                <table class="table is-fullwidth">
                    <tbody>
                        {
                            visible_transactions.map(|(confirmations, tx)| {
                                html! {
                                    <tr>
                                        <td>
                                            if confirmations < 3 {
                                                <span class="icon is-size-6 has-text-warning">
                                                    { format!("{}/3", confirmations) }
                                                </span>
                                            } else {
                                                <span class="icon has-text-success">
                                                // <span class="icon has-text-danger">
                                                    <i class="fas fa-circle"></i>
                                                </span>
                                            }
                                        </td>
                                        <td>
                                            <span class="has-text-grey-light is-family-code">
                                                { &tx.from }
                                            </span>
                                        </td>
                                        <td>
                                            if confirmations <= 3 {
                                                <span class="has-text-warning has-text-weight-medium">
                                                    { format!("{:+}", to_coin(self.cache.value_of(tx))) }
                                                </span>
                                            } else {
                                                <span class="has-text-success has-text-weight-medium">
                                                // <span class="has-text-danger has-text-weight-medium">
                                                    { format!("{:+}", to_coin(self.cache.value_of(tx))) }
                                                </span>
                                            }
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
                                        { to_coin(value) }
                                    </span>
                                </td>
                            </tr>
                        }
                    </tbody>
                </table>
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
            </div>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(_) => self.cache.update(&self.sim.borrow()),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.cache.clear();
        self.cache.full_node = ctx.props().full_node;
        self.cache.update(&self.sim.borrow())
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct TransactionsCache {
    full_node: Option<Entity>,
    // monitored_addresses: Vec<Address>,
    tip: Option<BlockHeader>,
    txes_confirmed: VecDeque<(usize, Transaction)>,
    txids_unconfirmed: BTreeSet<Entity>,
    txes_unconfirmed: VecDeque<Transaction>,
}
impl TransactionsCache {
    fn new() -> Self {
        Self::default()
    }
    fn clear(&mut self) -> bool {
        let new_self = Self::new();
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
    fn total_value_confirmed(&self) -> i32 {
        self.txes_confirmed
            .iter()
            .map(|(height, tx)| {
                if *height < self.tip_height() {
                    self.value_of(tx)
                } else {
                    0
                }
            })
            .sum()
    }
    fn total_value_unconfirmed(&self) -> i32 {
        self.txes_confirmed
            .iter()
            .map(|(height, tx)| {
                if height + 1 > self.tip_height() {
                    self.value_of(tx)
                } else {
                    0
                }
            })
            .sum::<i32>()
            + self
                .txes_unconfirmed
                .iter()
                .map(|tx| self.value_of(tx))
                .sum::<i32>()
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
                self.clear()
            }
        } else {
            self.clear()
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
    fn rebuild_confirmed_from_tip(&mut self, sim: &Simulation, block_id: Entity) {
        self.txes_confirmed.clear();
        let mut next_block = Some(block_id);
        while let Some(block_id) = next_block {
            next_block = self.update_confirmed_by_one_block(sim, block_id)
        }
    }
    // FIXME why in "Cache"?
    fn is_relevant(&self, tx: &Transaction) -> bool {
        // TODO
        true
    }
    fn value_of(&self, tx: &Transaction) -> i32 {
        // TODO
        tx.amount as i32
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

fn to_coin(satoshis: i32) -> f64 {
    satoshis as f64 / 10f64.powi(8)
}

// TODO tests? Should be able to peak into the `Html?`
