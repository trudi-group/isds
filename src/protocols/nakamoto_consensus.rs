use super::*;
use simple_flooding::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct NakamotoConsensus {
    flooding: SimpleFlooding<Block>,
}
impl NakamotoConsensus {
    pub fn new() -> Self {
        Self {
            flooding: SimpleFlooding::new(),
        }
    }
}

impl Protocol for NakamotoConsensus {
    type MessagePayload = SimpleFloodingMessage<Block>;

    fn handle_message(
        &self,
        mut node: NodeInterface,
        underlay_message: UnderlayMessage,
        message_payload: Self::MessagePayload,
    ) -> Result<(), Box<dyn Error>> {
        node.get::<NakamotoNodeState>()
            .register_block(message_payload.0);
        self.flooding
            .handle_message(node, underlay_message, message_payload)
    }

    fn handle_poke(&self, mut node: NodeInterface) -> Result<(), Box<dyn Error>> {
        node.log("Got poked by god, so I found a new block!");
        let tip_hash = node.get::<NakamotoNodeState>().tip;
        let block = Block::new(tip_hash, &mut node.rng());
        node.get::<NakamotoNodeState>().register_block(block);
        simple_flooding::flood(&mut node, block);
        Ok(())
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Block {
    hash: Hash,
    hash_prev: Hash,
}
impl Block {
    pub fn new(hash_prev: Hash, rng: &mut impl Rng) -> Self {
        let mut hash = Hash::default();
        rng.fill_bytes(&mut hash);
        Self { hash, hash_prev }
    }
    pub fn hash(&self) -> Hash {
        self.hash
    }
}

pub type Hash = [u8; 32]; // 256 bit

pub fn to_number(hash: Hash) -> u32 {
    let mut result: u32 = 0;
    for i in 0..4 {
        result += (hash[0] as u32) * 2u32.pow(i);
    }
    result
}

#[derive(Debug, Clone, Default)]
pub struct NakamotoNodeState {
    all_blocks: HashMap<Hash, (usize, Block)>,
    tip: Hash,
    fork_tips: HashSet<Hash>,
}
impl NakamotoNodeState {
    fn register_block(&mut self, block: Block) -> bool {
        if self.all_blocks.contains_key(&block.hash) {
            false
        } else if block.hash_prev == self.tip {
            self.all_blocks
                .insert(block.hash, (self.height(self.tip) + 1, block));
            self.tip = block.hash;
            true
        } else if block.hash_prev == Hash::default()
            || self.all_blocks.contains_key(&block.hash_prev)
        {
            self.all_blocks
                .insert(block.hash, (self.height(block.hash_prev) + 1, block));
            self.fork_tips.remove(&block.hash_prev); // can very well fail
            self.fork_tips.insert(block.hash);
            if self.height(block.hash) > self.height(self.tip) {
                let old_tip = self.tip;
                self.tip = block.hash;
                self.fork_tips.remove(&block.hash);
                self.fork_tips.insert(old_tip);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn tip(&self) -> Hash {
        self.tip
    }
    pub fn fork_tips(&self) -> &HashSet<Hash> {
        &self.fork_tips
    }
    pub fn height(&self, block_hash: Hash) -> usize {
        if block_hash == Hash::default() {
            0 // "genesis block"
        } else {
            self.all_blocks.get(&block_hash).expect("Unkown block?!").0
        }
    }
    pub fn hash_prev(&self, block_hash: Hash) -> Option<Hash> {
        self.all_blocks.get(&block_hash).map(|b| b.1.hash_prev)
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
        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let node3 = sim.spawn_random_node();
        add_peer(&mut sim, node1, node2);
        add_peer(&mut sim, node2, node1);
        add_peer(&mut sim, node2, node3);
        add_peer(&mut sim, node3, node2);

        let mut node_logic = Box::new(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        sim.do_now(PokeNode(node1));
        sim.catch_up(&mut [&mut *node_logic], &mut [], 100.);

        let state1 = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(node1)
            .expect("No relevant node state stored?")
            .clone();

        let state2 = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(node2)
            .expect("No relevant node state stored?")
            .clone();

        let state3 = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(node3)
            .expect("No relevant node state stored?")
            .clone();

        assert_eq!(state1.tip, state2.tip);
        assert_eq!(state1.tip, state3.tip);
    }

    #[wasm_bindgen_test]
    fn forks_get_registered() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let node3 = sim.spawn_random_node();
        add_peer(&mut sim, node1, node2);
        add_peer(&mut sim, node2, node1);
        add_peer(&mut sim, node2, node3);
        add_peer(&mut sim, node3, node2);

        let mut node_logic = Box::new(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        sim.do_now(PokeNode(node1));
        sim.catch_up(&mut [&mut *node_logic], &mut [], 100.);

        sim.do_now(PokeNode(node1));
        sim.do_now(PokeNode(node3));
        sim.catch_up(&mut [&mut *node_logic], &mut [], 100.);

        let state1 = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(node1)
            .expect("No relevant node state stored?")
            .clone();

        let state3 = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(node3)
            .expect("No relevant node state stored?")
            .clone();

        assert_ne!(state1.tip, state3.tip);

        let fork_tip_1 = state1
            .fork_tips
            .into_iter()
            .next()
            .expect("No forks registered?!");
        assert_eq!(fork_tip_1, state3.tip);
    }

    #[wasm_bindgen_test]
    fn forks_get_resolved() {
        let mut sim = Simulation::new();
        let node1 = sim.spawn_random_node();
        let node2 = sim.spawn_random_node();
        let node3 = sim.spawn_random_node();
        add_peer(&mut sim, node1, node2);
        add_peer(&mut sim, node2, node1);
        add_peer(&mut sim, node2, node3);
        add_peer(&mut sim, node3, node2);

        let mut node_logic = Box::new(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        sim.do_now(PokeNode(node1));
        sim.catch_up(&mut [&mut *node_logic], &mut [], 100.);

        sim.do_now(PokeNode(node1));
        sim.do_now(PokeNode(node3));
        sim.catch_up(&mut [&mut *node_logic], &mut [], 100.);

        sim.do_now(PokeNode(node1));
        sim.catch_up(&mut [&mut *node_logic], &mut [], 100.);

        let state1 = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(node1)
            .expect("No relevant node state stored?")
            .clone();

        let state3 = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(node3)
            .expect("No relevant node state stored?")
            .clone();

        assert_eq!(state1.tip, state3.tip);
    }

    #[wasm_bindgen_test]
    fn in_perfect_case_all_stored_blocks_are_connected_to_genesis() {
        let mut sim = Simulation::new();
        sim.do_now(SpawnRandomNodes(8));
        sim.do_now(MakeDelaunayNetwork);

        let mut node_logic = Box::new(InvokeProtocolForAllNodes(NakamotoConsensus::default()));

        sim.catch_up(&mut [&mut *node_logic], &mut [], 1.);

        for _ in 0..20 {
            sim.do_now(PokeMultipleRandomNodes(1));
            sim.catch_up(&mut [&mut *node_logic], &mut [], 100.);
        }

        let tested_node = sim.pick_random_node().unwrap();
        let state = sim
            .world
            .query_one_mut::<&NakamotoNodeState>(tested_node)
            .expect("No relevant node state stored?");

        let mut remaining_blocks = state.all_blocks.clone();

        let mut queue = vec![state.tip];
        queue.extend(state.fork_tips.clone().into_iter());

        while !queue.is_empty() {
            let block_hash = queue.pop().unwrap();
            if !state.all_blocks.contains_key(&block_hash) && block_hash != Hash::default() {
                panic!("Block not connected to genesis hash!");
            }
            if let Entry::Occupied(block_entry) = remaining_blocks.entry(block_hash) {
                queue.push(block_entry.get().1.hash_prev);
                block_entry.remove();
            }
        }
        assert!(remaining_blocks.is_empty());
    }
}
