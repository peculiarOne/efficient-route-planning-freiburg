use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

#[cfg(test)]
use serde_json::Result;

pub type OSMNodeId = u64;
pub type NodeIndex = usize;
pub type OSMWayId = u64;

pub const DEGREE_CONV: f64 = 10_000_000.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: OSMNodeId,
    pub latitude: i32,
    pub longitude: i32,
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
impl Node {
    pub fn new(id: OSMNodeId, lat: f64, long: f64) -> Node {
        Node {
            id: id,
            latitude: degrees_to_i32(lat),
            longitude: degrees_to_i32(long),
        }
    }

    pub fn lat_long_f64(&self) -> (f64, f64) {
        (self.latitude as f64 / DEGREE_CONV, self.longitude as f64 / DEGREE_CONV)
    }
}

pub fn degrees_to_i32(degrees: f64) -> i32 {
    (degrees * DEGREE_CONV) as i32
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Arc<T> {
    pub head_node: T,
    pub distance: u64,
    pub cost: u64,
    pub part_of_way: OSMWayId,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct WayInfo {
    pub id: OSMWayId,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkBuilder {
    all_nodes: HashMap<OSMNodeId, Node>,
    used_nodes: HashSet<OSMNodeId>,
    way_info: HashMap<OSMWayId, WayInfo>,
    pub adjacent_arcs: HashMap<OSMNodeId, Vec<Arc<OSMNodeId>>>,
}
impl NetworkBuilder {
    pub fn new() -> NetworkBuilder {
        NetworkBuilder {
            all_nodes: HashMap::new(),
            used_nodes: HashSet::new(),
            way_info: HashMap::new(),
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

    pub fn insert_way_info(&mut self, way_info: WayInfo) {
        self.way_info.insert(way_info.id, way_info);
    }

    pub fn total_arcs(&self) -> usize {
        self.adjacent_arcs.values().map(|v| v.len()).sum()
    }

    pub fn build_network(self) -> Option<Network> {
        println!("build_network");

        println!("total adjacent_arcs keys {}, total used nodes {}", self.adjacent_arcs.len(), self.used_nodes.len());

        let nodes_to_keep: Vec<&OSMNodeId> = if self.used_nodes.is_empty() { self.adjacent_arcs.keys().collect() } else { self.used_nodes.iter().collect() };
        let node_vec: Vec<Node> = nodes_to_keep.iter().map(|id| self.all_nodes.get(id).unwrap().clone()).collect();
        let with_index: HashMap<OSMNodeId, NodeIndex> = node_vec.iter().map(|n| n.id).zip(0..).collect();

        // build NodeIndex ordered graphs
        let mut forward_graph: Vec<Vec<Arc<NodeIndex>>> = vec![];
        for node in node_vec.iter() {
            let adjacent = self.adjacent_arcs.get(&node.id);
            let fwd_arcs: Vec<Arc<NodeIndex>> = match adjacent {
                Some(arcs) => {
                    arcs.iter().map(|a| {
                        let head_node_index = with_index.get(&a.head_node)
                        .and_then(|index| Some(Arc { 
                            head_node: *index,  
                            distance: a.distance,
                            cost: a.cost,
                            part_of_way: a.part_of_way,
                            })).unwrap();
                            head_node_index
                    }).collect()
                },
                None => vec![]
            };
            forward_graph.push(fwd_arcs);
        };

        Some(Network {
            node_indexes: with_index,
            nodes: node_vec,
            forward_graph: forward_graph,
            reverse_graph: vec![],
            way_info: self.way_info,
        })
    }

    #[cfg(test)]
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
    }
}

#[derive(Debug)]
pub struct Network {
    pub node_indexes: HashMap<OSMNodeId, NodeIndex>, 
    pub forward_graph: Vec<Vec<Arc<NodeIndex>>>,
    reverse_graph: Vec<Vec<Arc<NodeIndex>>>,
    nodes: Vec<Node>,
    way_info: HashMap<OSMWayId, WayInfo>,
}

impl Network {
    pub fn get_node(&self, node_id: &OSMNodeId) -> Option<&Node> {
        self.node_indexes.get(node_id).and_then(|&index| self.nodes.get(index))
    }

    pub fn get_way_info(&self, arc: &Arc<NodeIndex>) -> Option<&WayInfo> {
        self.way_info.get(&arc.part_of_way)
    }

    pub fn arc_count(&self) -> usize {
        self.forward_graph.iter().map(|v| v.len()).sum()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn fwd_arcs_from_node(&self, node_id: &OSMNodeId) -> Option<&Vec<Arc<NodeIndex>>> {
        let res = self.node_indexes.get(node_id).and_then(|&index| self.forward_graph.get(index));
        res
    }
}

#[test]
fn serialize() {
    let mut network = NetworkBuilder::new();
    network.insert_node(Node::new(1, 54.1, 6.4));
    network.insert_node(Node::new(2, 54.9, 6.2));
    network.insert_way_info(WayInfo{ id: 1, name: Some("Foo Street".to_string()) });
    network.insert_arc(
        1,
        Arc {
            head_node: 2,
            distance: 1500,
            cost: 2,
            part_of_way: 1,
        },
    );

    let toml = serde_json::to_string(&network).unwrap();
    println!("{}", &toml);

    let n2: NetworkBuilder = serde_json::from_str(&toml).unwrap();

    assert_eq!(network, n2);
}

#[test]
fn build_network() {
        let network_json = r#"{
        "all_nodes":{
            "1": {"id": 1, "latitude": 0, "longitude": 0},
            "2": {"id": 2, "latitude": 0, "longitude": 0},
            "3": {"id": 3, "latitude": 0, "longitude": 0},
            "4": {"id": 4, "latitude": 0, "longitude": 0},
            "5": {"id": 5, "latitude": 0, "longitude": 0},
            "6": {"id": 6, "latitude": 0, "longitude": 0},
            "7": {"id": 7, "latitude": 0, "longitude": 0}
        },
        "used_nodes":[5, 4, 3, 2, 1],
        "way_info":{
            "12": { "id": 12, "name": "1->2" },
            "13": { "id": 13, "name": "1->3" },
            "14": { "id": 14, "name": "1->4" },
            "21": { "id": 21, "name": "2->1" },
            "23": { "id": 23, "name": "2->3" },
            "31": { "id": 31, "name": "3->1" },
            "32": { "id": 32, "name": "3->2" },
            "34": { "id": 34, "name": "3->4" },
            "35": { "id": 35, "name": "3->5" },
            "41": { "id": 41, "name": "4->1" },
            "43": { "id": 43, "name": "4->3" },
            "51": { "id": 51, "name": "5->1" },
            "53": { "id": 53, "name": "5->3" }
        },
        "adjacent_arcs":{
            "1": [{"head_node": 4, "distance": 4, "cost": 4, "part_of_way": 14}, {"head_node": 2, "distance": 5, "cost": 5, "part_of_way": 12},{"head_node": 3, "distance": 2, "cost": 2, "part_of_way": 13}],
            "2": [{"head_node": 1, "distance": 5, "cost": 5, "part_of_way": 21}, {"head_node": 3, "distance": 4, "cost": 4, "part_of_way": 23},{"head_node": 5, "distance": 8, "cost": 8, "part_of_way": 25}],
            "3": [{"head_node": 1, "distance": 2, "cost": 2, "part_of_way": 31}, {"head_node": 2, "distance": 4, "cost": 4, "part_of_way": 32},{"head_node": 5, "distance": 2, "cost": 2, "part_of_way": 35},{"head_node": 4, "distance": 3, "cost": 3, "part_of_way": 34}],
            "4": [{"head_node": 1, "distance": 4, "cost": 4, "part_of_way": 41}, {"head_node": 3, "distance": 3, "cost": 3, "part_of_way": 43}],
            "5": [{"head_node": 2, "distance": 8, "cost": 8, "part_of_way": 51}, {"head_node": 3, "distance": 2, "cost": 2, "part_of_way": 53}]
        }
    }
    "#;

    let builder = NetworkBuilder::from_json(network_json).unwrap();
    let network = builder.clone().build_network().unwrap();
    println!("build network {:?}", network);

    assert_eq!(5, network.node_count());
    assert_eq!(builder.total_arcs(), network.arc_count());

    let node = network.get_node(&5);
    assert!(node.is_some());
    assert_eq!(5, node.unwrap().id);

}
