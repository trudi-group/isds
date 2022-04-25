use hecs::QueryItem;
use std::collections::BTreeSet;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockHeader {
    /// Substitute for the block's hash. We don't want to deal with the complexity of actual block
    /// hashes.
    pub id: Entity,
    /// `None` only for first block.
    pub id_prev: Option<Entity>,
    /// Not usually part of header but handy for us here.
    pub height: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BlockContents(BTreeSet<Entity>);
impl BlockContents {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> std::collections::btree_set::Iter<Entity> {
        self.0.iter()
    }
}
impl FromIterator<Entity> for BlockContents {
    fn from_iter<T: IntoIterator<Item = Entity>>(iter: T) -> Self {
        Self(BTreeSet::from_iter(iter))
    }
}
impl IntoIterator for BlockContents {
    type Item = Entity;
    type IntoIter = std::collections::btree_set::IntoIter<Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub value: u64,
}

pub type Address = String;
pub const TOSHIS_PER_COIN: u64 = 10_u64.pow(8);

pub fn coins_from(toshis: i64) -> f64 {
    toshis as f64 / TOSHIS_PER_COIN as f64
}

pub fn toshis_from(coins: f64) -> i64 {
    (coins * TOSHIS_PER_COIN as f64) as i64
}

impl<'a> NodeInterface<'a> {
    /// Registers a transaction in the global database, where it is immutable via the node
    /// interface.
    pub fn spawn_transaction(&mut self, from: Address, to: Address, value: u64) -> Entity {
        self.sim.world.spawn((Transaction { from, to, value },))
    }
    pub fn get_transaction(&mut self, tx_id: Entity) -> Option<QueryItem<&Transaction>> {
        self.sim.world.query_one_mut::<&Transaction>(tx_id).ok()
    }
    /// Registers a block in the global database, where it is immutable via the node interface.
    /// Set `id_prev` to `None` if this will be the first block in a chain (after the virtual
    /// genesis block).
    pub fn spawn_block(
        &mut self,
        id_prev: Option<Entity>,
        contents: impl IntoIterator<Item = Entity>,
    ) -> BlockHeader {
        let height = if let Some(id_prev) = id_prev {
            self.get_block_header(id_prev)
                .expect("No block exists at `id_prev`!")
                .height
                + 1
        } else {
            1
        };
        let id = self.sim.world.reserve_entity();
        let block_header = BlockHeader {
            id,
            id_prev,
            height,
        };
        let block_contents: BlockContents = contents.into_iter().collect();
        self.sim
            .world
            .insert(id, (block_header, block_contents))
            .unwrap();
        block_header
    }
    pub fn get_block(
        &mut self,
        block_id: Entity,
    ) -> Option<QueryItem<(&BlockHeader, &BlockContents)>> {
        self.sim
            .world
            .query_one_mut::<(&BlockHeader, &BlockContents)>(block_id)
            .ok()
    }
    pub fn get_block_header(&mut self, block_id: Entity) -> Option<QueryItem<&BlockHeader>> {
        self.sim.world.query_one_mut::<&BlockHeader>(block_id).ok()
    }
    pub fn get_block_contents(&mut self, block_id: Entity) -> Option<QueryItem<&BlockContents>> {
        self.sim
            .world
            .query_one_mut::<&BlockContents>(block_id)
            .ok()
    }
}

#[cfg(test)]
#[allow(clippy::redundant_clone)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    impl Transaction {
        fn new(from: Address, to: Address, value: u64) -> Self {
            Self { from, to, value }
        }
    }

    #[wasm_bindgen_test]
    fn transactions_are_spawned_and_gettable() {
        let mut sim = Simulation::new();
        sim.do_now(SpawnRandomNodes(1));
        sim.catch_up(10.);

        let node_id = sim.pick_random_node().unwrap();
        let mut node = sim.node_interface(node_id);

        let a1 = "Alice".to_string();
        let a2 = "Bob".to_string();
        let a3 = "Charlie".to_string();

        let tx_1_id = node.spawn_transaction(a1.clone(), a2.clone(), 123);
        let tx_2_id = node.spawn_transaction(a2.clone(), a3.clone(), 155);

        let expected_tx_1 = Transaction::new(a1.clone(), a2.clone(), 123);
        let expected_tx_2 = Transaction::new(a2.clone(), a3.clone(), 155);

        assert_eq!(Some(expected_tx_1), node.get_transaction(tx_1_id).cloned());
        assert_eq!(Some(expected_tx_2), node.get_transaction(tx_2_id).cloned());
    }

    #[wasm_bindgen_test]
    fn block_headers_are_spawned_and_gettable() {
        let mut sim = Simulation::new();
        sim.do_now(SpawnRandomNodes(1));
        sim.catch_up(10.);

        let node_id = sim.pick_random_node().unwrap();
        let mut node = sim.node_interface(node_id);

        let block_1_header = node.spawn_block(None, []);
        let block_2_header = node.spawn_block(Some(block_1_header.id), []);

        assert_eq!(
            Some(block_1_header),
            node.get_block_header(block_1_header.id).copied()
        );
        assert_eq!(
            block_2_header,
            *node.get_block(block_2_header.id).unwrap().0
        );
    }

    #[wasm_bindgen_test]
    fn spawn_block_sets_height_correctly() {
        let mut sim = Simulation::new();
        sim.do_now(SpawnRandomNodes(1));
        sim.catch_up(10.);

        let node_id = sim.pick_random_node().unwrap();
        let mut node = sim.node_interface(node_id);

        let block_1_header = node.spawn_block(None, []);
        let block_2_header = node.spawn_block(Some(block_1_header.id), []);

        assert_eq!(1, block_1_header.height);
        assert_eq!(2, block_2_header.height);
    }

    #[wasm_bindgen_test]
    fn block_contents_are_spawned_and_gettable() {
        let mut sim = Simulation::new();
        sim.do_now(SpawnRandomNodes(1));
        sim.catch_up(10.);

        let node_id = sim.pick_random_node().unwrap();
        let mut node = sim.node_interface(node_id);

        let a1 = "Alice".to_string();
        let a2 = "Bob".to_string();
        let a3 = "Charlie".to_string();

        let tx_1_id = node.spawn_transaction(a1.clone(), a2.clone(), 123);
        let tx_2_id = node.spawn_transaction(a2.clone(), a3.clone(), 155);

        let block_1_header = node.spawn_block(None, []);
        let block_2_header = node.spawn_block(Some(block_1_header.id), vec![tx_1_id, tx_2_id]);

        let expected_block_1_contents = BlockContents::new();
        let expected_block_2_contents = vec![tx_1_id, tx_2_id]
            .into_iter()
            .collect::<BlockContents>();

        assert_eq!(
            Some(expected_block_1_contents),
            node.get_block_contents(block_1_header.id).cloned()
        );
        assert_eq!(
            expected_block_2_contents,
            node.get_block(block_2_header.id).unwrap().1.clone()
        );
    }
}
