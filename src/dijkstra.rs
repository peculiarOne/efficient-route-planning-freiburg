extern crate env_logger;
extern crate log;

use crate::network::{Network, NodeId};

use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Entry {
    node: NodeId,
    cost: u64,
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cost.cmp(&self.cost))
    }
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.cost == other.cost
    }
}
impl Eq for Entry {}

pub fn run_dijsktra(
    source: NodeId,
    target: NodeId,
    graph: &Network,
    max_distance: u64,
) -> Option<Entry> {
    let mut costs: HashMap<NodeId, u64> = HashMap::new();

    let mut heap = BinaryHeap::new();

    // best_costs.insert(source, 0);
    heap.push(Entry {
        node: source,
        cost: 0,
    });
    costs.insert(source, 0);

    let mut count = 0;
    while let Some(entry) = heap.pop() {
        if max_distance > 0 && entry.cost > max_distance {
            break;
        }

        count += 1;
        if count % 20 == 0 {
            print_progress(&entry, &costs, &heap)
        }
        debug!("assessing node {}", entry.node);
        print_heap(&heap);

        if entry.node == target {
            return Some(entry);
        }

        // if is_best_cost(&entry, &costs) {
        {
            // println!("found best cost of {} to node {}", entry.cost, entry.node);
            let x = graph.adjacent_arcs.get(&entry.node);

            x.map(|arcs| {
                for arc in arcs {
                    let arc_entry = Entry {
                        node: arc.head_node,
                        cost: arc.cost + entry.cost,
                    };
                    // println!("neighbouring arc to {}", arc.head_node);

                    if is_best_cost(&arc_entry, &costs) {
                        costs.insert(arc_entry.node, arc_entry.cost);
                        heap.push(arc_entry);
                    }
                }
            });
        }
    }
    None
}

fn print_progress(
    currentEntry: &Entry,
    best_costs: &HashMap<NodeId, u64>,
    heap: &BinaryHeap<Entry>,
) {
    println!("--");
    println!(
        "assessing node {} with cost {}",
        currentEntry.node, currentEntry.cost
    );
    println!("{} entries still in heap", heap.len());
    println!("{} entries in best_cost", best_costs.len());
}

fn print_heap(heap: &BinaryHeap<Entry>) {
    debug!(
        "current heap <{}>",
        heap.iter()
            .map(|e| format!("(n:{}, c:{})", e.node, e.cost))
            .collect::<Vec<String>>()
            .join(", ")
    );
}

fn is_best_cost(entry: &Entry, best_costs: &HashMap<NodeId, u64>) -> bool {
    let maybe_best = best_costs.get(&entry.node);
    match maybe_best {
        Some(existing_best) if entry.cost < *existing_best => true,
        None => true,
        _ => false,
    }
}

#[test]
fn test_best_cost() {
    assert_eq!(
        true,
        is_best_cost(&Entry { node: 1, cost: 10 }, &HashMap::new())
    );

    let mut best_costs = HashMap::new();
    best_costs.insert(1, 9);

    assert_eq!(true, is_best_cost(&Entry { node: 1, cost: 8 }, &best_costs));
    assert_eq!(
        false,
        is_best_cost(&Entry { node: 1, cost: 11 }, &best_costs)
    );
}

#[test]
fn test_dijsktra() {
    let dummy_network = make_dummy_network();

    do_disjktra(&dummy_network, 4, 2, 7);
    do_disjktra(&dummy_network, 1, 4, 4);
    do_disjktra(&dummy_network, 1, 3, 2);
    do_disjktra(&dummy_network, 1, 5, 4);
    do_disjktra(&dummy_network, 2, 5, 6);
    do_disjktra(&dummy_network, 5, 4, 5);
}

fn do_disjktra(network: &Network, source: NodeId, destination: NodeId, expected_cost: u64) {
    let maybe_entry = run_dijsktra(source, destination, network, 0);

    assert!(maybe_entry.is_some());
    assert_eq!(expected_cost, maybe_entry.unwrap().cost);
}

fn make_dummy_network() -> Network {
    let network_json = r#"{
        "nodes":{},
        "adjacent_arcs":{
            "1": [{"head_node": 4, "distance": 4, "cost": 4}, {"head_node": 2, "distance": 5, "cost": 5},{"head_node": 3, "distance": 2, "cost": 2}],
            "2": [{"head_node": 1, "distance": 5, "cost": 5}, {"head_node": 3, "distance": 4, "cost": 4},{"head_node": 5, "distance": 8, "cost": 8}],
            "3": [{"head_node": 1, "distance": 2, "cost": 2}, {"head_node": 2, "distance": 4, "cost": 4},{"head_node": 5, "distance": 2, "cost": 2},{"head_node": 4, "distance": 3, "cost": 3}],
            "4": [{"head_node": 1, "distance": 4, "cost": 4}, {"head_node": 3, "distance": 3, "cost": 3}],
            "5": [{"head_node": 2, "distance": 8, "cost": 8}, {"head_node": 3, "distance": 2, "cost": 2}]
        }
    }
    "#;
    Network::from_json(network_json).unwrap()
}
