use super::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct RandomWalks {
    walks_ttl: usize,
}
impl RandomWalks {
    pub fn new(walks_ttl: usize) -> Self {
        Self { walks_ttl }
    }
}
impl Protocol for RandomWalks {
    type MessagePayload = RandomWalkMessage;
    fn handle_message(
        &self,
        mut node: NodeInterface,
        _: UnderlayMessage,
        message_payload: RandomWalkMessage,
    ) -> Result<(), Box<dyn Error>> {
        let ttl = message_payload.ttl;
        if ttl > 0 {
            random_step(&mut node, ttl)?;
        } else {
            node.log("A random walk ended!");
        }
        Ok(())
    }
    fn handle_poke(&self, mut node: NodeInterface) -> Result<(), Box<dyn Error>> {
        random_step(&mut node, self.walks_ttl)?;
        Ok(())
    }
}

pub fn random_step(node: &mut NodeInterface, current_ttl: usize) -> Result<Entity, String> {
    if let Some(dest) = random_peer(node) {
        Ok(node.send_message(dest, RandomWalkMessage::new(current_ttl - 1)))
    } else {
        Err("Couldn't find a suitable message destination. Not enough peers?".to_string())
    }
}

fn random_peer(node: &mut NodeInterface) -> Option<Entity> {
    let peers = node.get::<PeerSet>().0.clone(); // TODO: the `.clone()` here is not ideal
    peers.iter().choose(node.rng()).copied()
}

#[derive(Debug, Copy, Clone)]
pub struct RandomWalkMessage {
    pub ttl: usize,
}
impl RandomWalkMessage {
    pub fn new(ttl: usize) -> Self {
        Self { ttl }
    }
}
