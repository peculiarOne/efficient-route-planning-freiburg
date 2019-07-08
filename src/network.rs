use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[cfg(test)]
use serde_json::Result;

pub type OSMNodeId = u32;
pub type NodeIndex = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: OSMNodeId,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Arc<T> {
    pub head_node: T,
    pub distance: u64,
    pub cost: u64,
    pub part_of_way: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkBuilder {
    all_nodes: HashMap<OSMNodeId, Node>,
    used_nodes: HashSet<OSMNodeId>,
    pub adjacent_arcs: HashMap<OSMNodeId, Vec<Arc<OSMNodeId>>>,
}
impl NetworkBuilder {
    pub fn new() -> NetworkBuilder {
        NetworkBuilder {
            all_nodes: HashMap::new(),
            used_nodes: HashSet::new(),
            adjacent_arcs: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn from_json(json: &str) -> Result<NetworkBuilder> {
        serde_json::from_str(json)
    }

    pub fn insert_node(&mut self, node: Node) {
        self.all_nodes.insert(node.id, node);
    }

    pub fn get_node(&self, node_id: &OSMNodeId) -> Option<&Node> {
        self.all_nodes.get(node_id)
    }

    pub fn insert_arc(&mut self, start_node: OSMNodeId, arc: Arc<OSMNodeId>) {

        self.used_nodes.insert(arc.head_node);
        self.used_nodes.insert(start_node);

        let existing = self.adjacent_arcs.entry(start_node).or_insert(vec![]);
        existing.push(arc);

    }

    pub fn total_arcs(&self) -> usize {
        self.adjacent_arcs.values().map(|v| v.len()).sum()
    }

    pub fn build_network(&self) -> Option<Network> {

        println!("total adjacent_arcs keys {}, total used nodes {}", self.adjacent_arcs.len(), self.used_nodes.len());


        None
    }

    #[cfg(test)]
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }
}

pub struct Network {
    nodes: Vec<Node>,
    pub node_indexes: HashMap<OSMNodeId, NodeIndex>, 
    pub forward_graph: Vec<Vec<Arc<NodeIndex>>>,
    reverse_graph: Vec<Vec<Arc<NodeIndex>>>,
}

impl Network {
    pub fn get_node(&self, node_id: &OSMNodeId) -> Option<&Node> {
        self.node_indexes.get(node_id).and_then(|&index| self.nodes.get(index))
    }

    pub fn arc_count(&self) -> usize {
        self.forward_graph.iter().map(|v| v.len()).sum()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[test]
fn serialize() {
    let mut network = NetworkBuilder::new();
    network.insert_node(Node {
        id: 1,
        latitude: 54.1,
        longitude: 6.4,
    });
    network.insert_node(Node {
        id: 2,
        latitude: 54.9,
        longitude: 6.2,
    });
    network.insert_arc(
        1,
        Arc {
            head_node: 2,
            distance: 1500,
            cost: 2,
            part_of_way: Some("Foo Street".to_string()),
        },
    );

    let toml = serde_json::to_string(&network).unwrap();
    println!("{}", &toml);

    let n2: NetworkBuilder = serde_json::from_str(&toml).unwrap();

    assert_eq!(network, n2);
}
