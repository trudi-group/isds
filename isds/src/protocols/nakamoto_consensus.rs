use super::*;
use simple_flooding::*;
use std::collections::{BTreeSet, HashMap, HashSet};

use blockchain_types::*;

#[derive(Debug, Clone)]
pub struct BuildAndBroadcastTransaction {
    from: Address,
    to: Address,
    amount: u64,
}
impl BuildAndBroadcastTransaction {
    pub fn new(from: &str, to: &str, amount: u64) -> Self {
        let from = from.to_string();
        let to = to.to_string();
        Self { from, to, amount }
    }
}
impl EntityAction for BuildAndBroadcastTransaction {
    fn execute_for(&self, sim: &mut Simulation, entity: Entity) -> Result<(), Box<dyn Error>> {
        let mut node = sim.node_interface(entity);
        NakamotoConsensus::handle_new_transaction(
            &mut node,
            self.from.clone(),
            self.to.clone(),
            self.amount,
        )
    }
}

#[derive(Debug, Clone)]
pub struct MineBlock;
impl EntityAction for MineBlock {
    fn execute_for(&self, sim: &mut Simulation, entity: Entity) -> Result<(), Box<dyn Error>> {
        let mut node = sim.node_interface(entity);
        NakamotoConsensus::handle_mining_success(&mut node)
    }
}

#[derive(Debug, Default)]
pub struct NakamotoConsensus {
    flooding: SimpleFlooding<InventoryItem>,
}
impl NakamotoConsensus {
    pub fn new() -> Self {
        Self {
            flooding: SimpleFlooding::new(),
        }
    }
    fn handle_transaction(node: &mut NodeInterface, tx_id: Entity) -> Result<(), Box<dyn Error>> {
        node.get::<NakamotoNodeState>()
            .register_transaction_id(tx_id);
        Ok(())
    }
    fn handle_block(node: &mut NodeInterface, block_id: Entity) -> Result<(), Box<dyn Error>> {
        let (&block_header, block_contents) = node
            .get_block(block_id)
            .ok_or("Received a block that doesn't exist!")?;
        let block_contents = block_contents.clone();
        node.get::<NakamotoNodeState>()
            .register_block(block_header, block_contents);
        Ok(())
    }
    fn handle_new_transaction(
        node: &mut NodeInterface,
        from: Address,
        to: Address,
        amount: u64,
    ) -> Result<(), Box<dyn Error>> {
        node.log(&format!(
            "Building new transaction: {} toshis from {} to {}.",
            amount, from, to
        ));
        let tx_id = node.spawn_transaction(from, to, amount);
        node.get::<NakamotoNodeState>()
            .register_transaction_id(tx_id);
        SimpleFlooding::flood(node, InventoryItem::Transaction(tx_id));
        Ok(())
    }
    fn handle_mining_success(node: &mut NodeInterface) -> Result<(), Box<dyn Error>> {
        let tip = node.get::<NakamotoNodeState>().tip;
        let contents = node
            .get::<NakamotoNodeState>()
            .drain_unconfirmed_transactions();
        let block_header = node.spawn_block(tip, contents);
        let block_contents = node.get_block_contents(block_header.id).unwrap().clone();
        node.log(&format!(
            "Mined a new block of height {} that contains {} transactions.",
            block_header.height,
            block_contents.len()
        ));
        node.get::<NakamotoNodeState>()
            .register_block(block_header, block_contents);
        SimpleFlooding::flood(node, InventoryItem::Block(block_header.id));
        Ok(())
    }
    fn handle_peer_removed(mut node: NodeInterface, peer: Entity) -> Result<(), Box<dyn Error>> {
        SimpleFlooding::<InventoryItem>::forget_peer(&mut node, peer);
        Ok(())
    }
    fn handle_peer_added(node: &mut NodeInterface, peer: Entity) -> Result<(), Box<dyn Error>> {
        let all_blocks_sorted = node.get::<NakamotoNodeState>().known_blocks_sorted();
        SimpleFlooding::<InventoryItem>::flood_peer_with(
            node,
            peer,
            all_blocks_sorted.into_iter().map(InventoryItem::Block),
        );
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InventoryItem {
    Transaction(Entity),
    Block(Entity),
}

impl Protocol for NakamotoConsensus {
    type MessagePayload = SimpleFloodingMessage<InventoryItem>;

    fn handle_message(
        &self,
        mut node: NodeInterface,
        underlay_message: UnderlayMessage,
        message_payload: Self::MessagePayload,
    ) -> Result<(), Box<dyn Error>> {
        match message_payload.0 {
            InventoryItem::Transaction(tx_id) => {
                Self::handle_transaction(&mut node, tx_id)?;
            }
            InventoryItem::Block(block_id) => {
                Self::handle_block(&mut node, block_id)?;
            }
        }
        self.flooding
            .handle_message(node, underlay_message, message_payload)
    }

    fn handle_poke(&self, mut node: NodeInterface) -> Result<(), Box<dyn Error>> {
        Self::handle_mining_success(&mut node)
    }

    fn handle_peer_set_update(
        &self,
        mut node: NodeInterface,
        update: PeerSetUpdate,
    ) -> Result<(), Box<dyn Error>> {
        match update {
            PeerSetUpdate::PeerAdded(peer) => {
                Self::handle_peer_added(&mut node, peer)?;
            }
            PeerSetUpdate::PeerRemoved(peer) => {
                Self::handle_peer_removed(node, peer)?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct NakamotoNodeState {
    known_blocks: HashMap<Entity, BlockHeader>,
    tip: Option<Entity>,
    fork_tips: HashSet<Entity>,
    txes_unconfirmed: BTreeSet<Entity>,
    txes_confirmed: HashSet<Entity>,
}
impl NakamotoNodeState {
    /// Returns `true` if we have updated the tip of the blockchain.
    fn register_block(&mut self, header: BlockHeader, contents: BlockContents) -> bool {
        // Our simple logic here assumes that blocks always arrive in the same order.
        // Making this better might be a TODO.
        if self.known_blocks.contains_key(&header.id) {
            false
        } else if header.id_prev == self.tip {
            self.known_blocks.insert(header.id, header);
            self.register_new_tip(header.id, contents);
            true
        } else if header.id_prev == None {
            self.known_blocks.insert(header.id, header);
            self.fork_tips.insert(header.id);
            false
        } else if self.known_blocks.contains_key(&header.id_prev.unwrap()) {
            self.known_blocks.insert(header.id, header);
            self.fork_tips.remove(&header.id_prev.unwrap()); // will do nothing if it's a new fork
            self.fork_tips.insert(header.id);
            if header.height > self.tip_height() {
                let old_tip = self.tip.unwrap();
                self.register_new_tip(header.id, contents);
                self.fork_tips.remove(&header.id);
                self.fork_tips.insert(old_tip);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    fn register_new_tip(&mut self, block_id: Entity, block_contents: BlockContents) {
        self.tip = Some(block_id);
        for tx_id in block_contents.into_iter() {
            self.txes_unconfirmed.remove(&tx_id);
            self.txes_confirmed.insert(tx_id);
        }
    }
    fn register_transaction_id(&mut self, tx_id: Entity) {
        if !self.txes_confirmed.contains(&tx_id) {
            self.txes_unconfirmed.insert(tx_id);
        }
    }
    fn drain_unconfirmed_transactions(&mut self) -> impl IntoIterator<Item = Entity> {
        let mut tmp = BTreeSet::new();
        std::mem::swap(&mut self.txes_unconfirmed, &mut tmp);
        tmp
    }
    pub fn block_header(&self, block_id: Entity) -> Option<BlockHeader> {
        self.known_blocks.get(&block_id).copied()
    }
    pub fn tip(&self) -> Option<Entity> {
        self.tip
    }
    pub fn fork_tips(&self) -> &HashSet<Entity> {
        &self.fork_tips
    }
    pub fn height(&self, block_id: Option<Entity>) -> usize {
        if let Some(block_id) = block_id {
            self.known_blocks
                .get(&block_id)
                .expect("Requested height of unknown block.")
                .height
        } else {
            0
        }
    }
    pub fn tip_height(&self) -> usize {
        self.height(self.tip)
    }
    /// Returns the ids of all known blocks (forks included) sorted by their block height,
    /// smallest heights first.
    pub fn known_blocks_sorted(&self) -> Vec<Entity> {
        let mut blocks_heights: Vec<(usize, Entity)> = self
            .known_blocks
            .iter()
            .map(|(&block_id, &block)| (block.height, block_id))
            .collect();
        blocks_heights.sort_by(|a, b| a.0.cmp(&b.0));
        blocks_heights.into_iter().map(|(_, block)| block).collect()
    }
    pub fn txes_unconfirmed(&self) -> &BTreeSet<Entity> {
        &self.txes_unconfirmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::Entry;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn get_state(sim: &Simulation, node_id: Entity) -> NakamotoNodeState {
        sim.world
            .query_one::<&NakamotoNodeState>(node_id)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone()
    }

    #[wasm_bindgen_test]
    fn blocks_get_distributed() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let node3 = sim.spawn_random_node();
        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);
        sim.add_peer(node2, node3);
        sim.add_peer(node3, node2);

        sim.do_now(ForSpecific(node1, MineBlock));
        sim.catch_up(100.);

        let state1 = get_state(&mut sim, node1);
        let state2 = get_state(&mut sim, node2);
        let state3 = get_state(&mut sim, node3);

        assert_eq!(state1.tip, state2.tip);
        assert_eq!(state1.tip, state3.tip);
    }

    #[wasm_bindgen_test]
    fn transactions_get_distributed() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let node3 = sim.spawn_random_node();
        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);
        sim.add_peer(node2, node3);
        sim.add_peer(node3, node2);

        sim.do_now(ForSpecific(
            node1,
            BuildAndBroadcastTransaction::new("Alice", "Bob", 32),
        ));
        sim.catch_up(100.);

        let state1 = get_state(&mut sim, node1);
        let state2 = get_state(&mut sim, node2);
        let state3 = get_state(&mut sim, node3);

        assert!(!state1.txes_unconfirmed.is_empty());
        assert_eq!(state1.txes_unconfirmed, state2.txes_unconfirmed);
        assert_eq!(state1.txes_unconfirmed, state3.txes_unconfirmed);
    }

    #[wasm_bindgen_test]
    fn transactions_end_up_in_blocks() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);

        sim.do_now(ForSpecific(
            node1,
            BuildAndBroadcastTransaction::new("Alice", "Bob", 32),
        ));
        sim.catch_up(100.);

        sim.do_now(ForSpecific(node2, MineBlock));
        sim.catch_up(100.);

        let state1 = get_state(&mut sim, node1);
        let state2 = get_state(&mut sim, node2);

        assert!(state1.txes_unconfirmed.is_empty());
        assert!(state2.txes_unconfirmed.is_empty());
        assert!(!state1.txes_confirmed.is_empty());

        let block_id = state1.tip().unwrap();
        let block_contents = sim
            .world
            .query_one::<&BlockContents>(block_id)
            .unwrap()
            .get()
            .expect("Block contents do not exist?")
            .clone();

        assert!(block_contents.len() == 1);
    }

    #[wasm_bindgen_test]
    fn transactions_are_not_registered_if_already_confirmed() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);

        sim.do_now(ForSpecific(
            node1,
            BuildAndBroadcastTransaction::new("Alice", "Bob", 32),
        ));
        sim.do_now(ForSpecific(node1, MineBlock));
        sim.catch_up(100.);

        let mut state2 = get_state(&mut sim, node2);

        assert!(state2.txes_unconfirmed.is_empty());
        assert!(!state2.txes_confirmed.is_empty());

        let tx_id = state2.txes_confirmed.iter().cloned().next().unwrap();

        state2.register_transaction_id(tx_id);

        assert!(state2.txes_unconfirmed.is_empty());
    }

    #[wasm_bindgen_test]
    fn forks_get_registered() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let node3 = sim.spawn_random_node();
        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);
        sim.add_peer(node2, node3);
        sim.add_peer(node3, node2);

        sim.do_now(ForSpecific(node1, MineBlock));
        sim.catch_up(100.);

        sim.do_now(ForSpecific(node1, MineBlock));
        sim.do_now(ForSpecific(node3, MineBlock));
        sim.catch_up(100.);

        let state1 = get_state(&mut sim, node1);
        let state3 = get_state(&mut sim, node3);

        assert_ne!(state1.tip, state3.tip);

        let fork_tip_1 = state1
            .fork_tips
            .into_iter()
            .next()
            .expect("No forks registered?!");
        assert_eq!(fork_tip_1, state3.tip.unwrap());
    }

    #[wasm_bindgen_test]
    fn forks_get_resolved() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let node3 = sim.spawn_random_node();
        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);
        sim.add_peer(node2, node3);
        sim.add_peer(node3, node2);

        sim.do_now(ForSpecific(node1, MineBlock));
        sim.catch_up(100.);

        sim.do_now(ForSpecific(node1, MineBlock));
        sim.do_now(ForSpecific(node3, MineBlock));
        sim.catch_up(100.);

        sim.do_now(ForSpecific(node1, MineBlock));
        sim.catch_up(100.);

        let state1 = get_state(&mut sim, node1);
        let state3 = get_state(&mut sim, node3);

        assert_eq!(state1.tip, state3.tip);
    }

    #[wasm_bindgen_test]
    fn in_perfect_case_all_stored_blocks_are_connected_to_genesis() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        sim.do_now(SpawnRandomNodes(8));
        sim.do_now(MakeDelaunayNetwork);

        sim.catch_up(1.);

        for _ in 0..20 {
            sim.do_now(ForRandomNode(MineBlock));
            sim.catch_up(100.);
        }

        let tested_node = sim.pick_random_node().unwrap();
        let state = get_state(&mut sim, tested_node);

        let mut remaining_blocks = state.known_blocks.clone();

        let mut queue = vec![state.tip];
        queue.extend(state.fork_tips.clone().into_iter().map(|id| Some(id)));

        while !queue.is_empty() {
            let block_id = queue.pop().unwrap();
            if let Some(block_id) = block_id {
                if !state.known_blocks.contains_key(&block_id) {
                    panic!("Block not connected to genesis hash!");
                }
                if let Entry::Occupied(block_entry) = remaining_blocks.entry(block_id) {
                    queue.push(block_entry.get().id_prev);
                    block_entry.remove();
                }
            }
        }
        assert!(remaining_blocks.is_empty());
    }

    #[wasm_bindgen_test]
    fn nakamoto_consensus_recovers_from_splits() {
        let mut sim = Simulation::new();
        sim.add_event_handler(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();

        sim.do_now(ForSpecific(node1, MineBlock));
        sim.do_now(ForSpecific(node1, MineBlock));
        sim.do_now(ForSpecific(node1, MineBlock));

        sim.do_now(ForSpecific(node2, MineBlock));
        sim.do_now(ForSpecific(node2, MineBlock));

        sim.catch_up(10.);

        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);

        sim.catch_up(10.);

        let state1 = get_state(&mut sim, node1);
        let state2 = get_state(&mut sim, node2);

        assert_eq!(state1.height(state1.tip), state2.height(state2.tip));
        assert_eq!(state1.tip, state2.tip);
    }
}
