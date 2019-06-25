use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub type NodeId = u64;

pub struct Node {
    pub id: NodeId,
    pub latitude: f64,
    pub longitude: f64,
}
impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.id == other.id
    }
}
impl Eq for Node {}
impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Clone, Debug)]
pub struct Arc {
    pub head_node: NodeId,
    pub distance: f64,
    pub cost: u64,
}

pub struct Network {
    pub nodes: HashMap<NodeId, Node>,
    pub adjacent_arcs: HashMap<NodeId, Vec<Arc>>,
}
impl Network {
    pub fn new() -> Network {
        Network {
            nodes: HashMap::new(),
            adjacent_arcs: HashMap::new(),
        }
    }

    pub fn insert_node(&mut self, node: Node) {
        self.nodes.insert(node.id, node);
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&Node>{
        self.nodes.get(node_id)
    }

    pub fn insert_arc(&mut self, start_node: NodeId, arc: Arc) {
        let existing = self.adjacent_arcs.entry(start_node).or_insert(vec![]);
        existing.push(arc);
    }

    pub fn total_arcs(&self) -> usize {
        self.adjacent_arcs.values().map(|v| v.len()).sum()
    }
}
