// use network::{Network, NodeId};
use crate::network::{Network, NodeId};

use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::collections::HashMap;

struct Entry {
    node: NodeId,
    cost: u32,
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cost.cmp(&other.cost))
    }
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.cost == other.cost
    }
}
impl Eq for Entry {}

fn run_dijsktra(source: NodeId, dest: NodeId, graph: Network) {
    let mut best_costs: HashMap<NodeId, u32> = HashMap::new();

    let mut heap = BinaryHeap::new();

    best_costs.insert(source, 0);
    heap.push(Entry {
        node: source,
        cost: 0,
    });

    while let Some(x) = heap.pop() {}
}
