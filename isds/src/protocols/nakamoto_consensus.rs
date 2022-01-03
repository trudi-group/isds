use super::*;
use simple_flooding::*;
use std::collections::{HashMap, HashSet};

use blockchain_types::*;

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
            InventoryItem::Transaction(_) => {
                todo!();
            }
            InventoryItem::Block(block_id) => {
                let block_header = *node
                    .get_block_header(block_id)
                    .expect("Received a block that doesn't exist!");
                node.get::<NakamotoNodeState>().register_block(block_header);
            }
        }
        self.flooding
            .handle_message(node, underlay_message, message_payload)
    }

    fn handle_poke(&self, mut node: NodeInterface) -> Result<(), Box<dyn Error>> {
        node.log("Got poked by god, so I found a new block!");
        let tip = node.get::<NakamotoNodeState>().tip;
        let block_header = node.spawn_block(tip, []);
        node.get::<NakamotoNodeState>().register_block(block_header);
        SimpleFlooding::flood(&mut node, InventoryItem::Block(block_header.id));
        Ok(())
    }

    fn handle_peer_set_update(
        &self,
        mut node: NodeInterface,
        update: PeerSetUpdate,
    ) -> Result<(), Box<dyn Error>> {
        match update {
            PeerSetUpdate::PeerAdded(peer) => {
                let all_blocks_sorted = node.get::<NakamotoNodeState>().known_blocks_sorted();
                SimpleFlooding::<InventoryItem>::flood_peer_with(
                    &mut node,
                    peer,
                    all_blocks_sorted.into_iter().map(InventoryItem::Block),
                )
            }
            PeerSetUpdate::PeerRemoved(peer) => {
                SimpleFlooding::<InventoryItem>::forget_peer(&mut node, peer)
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
}
impl NakamotoNodeState {
    /// Returns `true` if we have updated the tip of the blockchain.
    fn register_block(&mut self, block: BlockHeader) -> bool {
        // Our simple logic here assumes that blocks always arrive in the same order.
        // Making this better might be a TODO.
        if self.known_blocks.contains_key(&block.id) {
            false
        } else if block.id_prev == self.tip {
            self.known_blocks.insert(block.id, block);
            self.tip = Some(block.id);
            true
        } else if block.id_prev == None {
            self.known_blocks.insert(block.id, block);
            self.fork_tips.insert(block.id);
            false
        } else if self.known_blocks.contains_key(&block.id_prev.unwrap()) {
            self.known_blocks.insert(block.id, block);
            self.fork_tips.remove(&block.id_prev.unwrap()); // will do nothing if it's a new fork
            self.fork_tips.insert(block.id);
            if block.height > self.tip_height() {
                let old_tip = self.tip.unwrap();
                self.tip = Some(block.id);
                self.fork_tips.remove(&block.id);
                self.fork_tips.insert(old_tip);
                true
            } else {
                false
            }
        } else {
            false
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::Entry;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

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

        sim.do_now(PokeSpecificNode(node1));
        sim.catch_up(100.);

        let state1 = sim
            .world
            .query_one::<&NakamotoNodeState>(node1)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

        let state2 = sim
            .world
            .query_one::<&NakamotoNodeState>(node2)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

        let state3 = sim
            .world
            .query_one::<&NakamotoNodeState>(node3)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

        assert_eq!(state1.tip, state2.tip);
        assert_eq!(state1.tip, state3.tip);
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

        sim.do_now(PokeSpecificNode(node1));
        sim.catch_up(100.);

        sim.do_now(PokeSpecificNode(node1));
        sim.do_now(PokeSpecificNode(node3));
        sim.catch_up(100.);

        let state1 = sim
            .world
            .query_one::<&NakamotoNodeState>(node1)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

        let state3 = sim
            .world
            .query_one::<&NakamotoNodeState>(node3)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

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

        sim.do_now(PokeSpecificNode(node1));
        sim.catch_up(100.);

        sim.do_now(PokeSpecificNode(node1));
        sim.do_now(PokeSpecificNode(node3));
        sim.catch_up(100.);

        sim.do_now(PokeSpecificNode(node1));
        sim.catch_up(100.);

        let state1 = sim
            .world
            .query_one::<&NakamotoNodeState>(node1)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

        let state3 = sim
            .world
            .query_one::<&NakamotoNodeState>(node3)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

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
            sim.do_now(ForRandomNode(PokeNode));
            sim.catch_up(100.);
        }

        let tested_node = sim.pick_random_node().unwrap();
        let state = sim
            .world
            .query_one::<&NakamotoNodeState>(tested_node)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

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

        sim.do_now(PokeSpecificNode(node1));
        sim.do_now(PokeSpecificNode(node1));
        sim.do_now(PokeSpecificNode(node1));

        sim.do_now(PokeSpecificNode(node2));
        sim.do_now(PokeSpecificNode(node2));

        sim.catch_up(10.);

        sim.add_peer(node1, node2);
        sim.add_peer(node2, node1);

        sim.catch_up(10.);

        let state1 = sim
            .world
            .query_one::<&NakamotoNodeState>(node1)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

        let state2 = sim
            .world
            .query_one::<&NakamotoNodeState>(node2)
            .unwrap()
            .get()
            .expect("No relevant node state stored?")
            .clone();

        assert_eq!(state1.height(state1.tip), state2.height(state2.tip));
        assert_eq!(state1.tip, state2.tip);
    }
}
