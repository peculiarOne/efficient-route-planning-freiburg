extern crate env_logger;
extern crate log;

use crate::network::{Network, NetworkBuilder, NodeIndex, OSMNodeId};

use std::cmp::{Ord, Ordering};
use std::collections::BinaryHeap;
use std::collections::HashMap;

const DEBUG: bool = false;
const REPORT_HEAP: bool = false;

#[derive(Clone, Debug)]
pub struct Entry<'a> {
    node: NodeIndex,
    pub cost: u64,
    arc_name: Option<&'a str>,
    prev_entry: Option<Box<Entry<'a>>>,
}
impl<'a> Ord for Entry<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}
impl<'a> PartialOrd for Entry<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cost.cmp(&self.cost))
    }
}
impl<'a> PartialEq for Entry<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.cost == other.cost
    }
}
impl<'a> Eq for Entry<'a> {}

impl<'a> Entry<'a> {
    pub fn report_traversed_ways(&self) -> String {
        let mut prev_entries = vec![];
        let mut prev = &self.prev_entry;

        while prev.is_some() {
            match prev {
                Some(p) => {
                    prev_entries.push(p);
                    prev = &p.prev_entry;
                }
                None => (),
            }
        }

        let mut arc_names: Vec<&str> = prev_entries
            .iter()
            .map(|e| e.arc_name.unwrap_or("unknown"))
            .collect();
        arc_names.dedup();
        arc_names.reverse();
        let ways = arc_names.join("->");
        ways
    }
}

pub fn run_dijsktra(
    source: OSMNodeId,
    target: OSMNodeId,
    network: &Network,
    max_distance: u64,
    trace_path: bool,
) -> Option<Entry> {
    if network.get_node(&target).is_none() {
        println!("!!run_dijkstra. target not in network")
    }

    let mut costs: HashMap<NodeIndex, u64> = HashMap::new();

    let mut heap = BinaryHeap::new();

    let maybe_source_index = network.node_indexes.get(&source);
    let maybe_target_index = network.node_indexes.get(&target);
    let (source_index, target_index) = match (maybe_source_index, maybe_target_index) {
        (Some(s), Some(t)) => (*s,*t),
        _ => {
            println!("dijkstra. Couldn't find source & target indexes");
            return None
        }
    };

    // best_costs.insert(source, 0);
    heap.push(Entry {
        node: source_index,
        cost: 0,
        arc_name: None,
        prev_entry: None,
    });
    costs.insert(source_index, 0);

    let mut count = 0;
    while let Some(entry) = heap.pop() {
        if max_distance > 0 && entry.cost > max_distance {
            break;
        }

        if DEBUG {
            count += 1;
            if count % 1 == 0 {
                print_progress(&entry, &costs, &heap)
            }
            if REPORT_HEAP {
                print_heap(&heap)
            }
        }

        if entry.node == target_index {
            println!("dijkstra. finished");
            return Some(entry);
        }

        let arcs = &network.forward_graph[entry.node];
        if DEBUG { println!("forward arcs from {}, {:?}", entry.node, arcs.iter())}

            for arc in arcs {
                let arc_name = if trace_path {
                    // arc.part_of_way.as_ref().map(|s| s.as_str())
                    network.get_way_info(&arc).and_then(|i| i.name.as_ref().map(String::as_str))
                } else {
                    None
                };
                let prev_entry = if trace_path {
                    Some(Box::new(entry.clone()))
                } else {
                    None
                };
                let arc_entry = Entry {
                    node: arc.head_node,
                    cost: arc.cost + entry.cost,
                    arc_name: arc_name,
                    prev_entry: prev_entry,
                };
                if DEBUG && REPORT_HEAP {
                    println!("\tneighbouring arc to {}, {:?}", arc.head_node, arc_name)
                }

                if is_best_cost(&arc_entry, &costs) {
                    costs.insert(arc_entry.node, arc_entry.cost);
                    heap.push(arc_entry);
                }
        };
    }
    None
}

fn print_progress(
    current_entry: &Entry,
    best_costs: &HashMap<NodeIndex, u64>,
    heap: &BinaryHeap<Entry>,
) {
    println!("--");
    println!(
        "assessing node {} with cost {}",
        current_entry.node, current_entry.cost
    );
    println!("{} entries still in heap", heap.len());
    println!("{} entries in best_cost", best_costs.len());
}

fn print_heap(heap: &BinaryHeap<Entry>) {
    println!(
        "current heap <{}>",
        heap.iter()
            .map(|e| format!("(n:{}, c:{})", e.node, e.cost))
            .collect::<Vec<String>>()
            .join(", ")
    );
}

fn is_best_cost(entry: &Entry, best_costs: &HashMap<NodeIndex, u64>) -> bool {
    let maybe_best = best_costs.get(&entry.node);
    match maybe_best {
        Some(existing_best) if entry.cost < *existing_best => true,
        None => true,
        _ => false,
    }
}

#[cfg(test)]
mod dijkstra_test {
    use super::*;

    #[test]
    fn test_best_cost() {
        assert_eq!(
            true,
            is_best_cost(
                &Entry {
                    node: 1,
                    cost: 10,
                    arc_name: None,
                    prev_entry: None
                },
                &HashMap::new()
            )
        );

        let mut best_costs = HashMap::new();
        best_costs.insert(1, 9);

        assert_eq!(
            true,
            is_best_cost(
                &Entry {
                    node: 1,
                    cost: 8,
                    arc_name: None,
                    prev_entry: None
                },
                &best_costs
            )
        );
        assert_eq!(
            false,
            is_best_cost(
                &Entry {
                    node: 1,
                    cost: 11,
                    arc_name: None,
                    prev_entry: None
                },
                &best_costs
            )
        );
    }

    #[test]
    fn test_dijsktra() {
        let dummy_network = make_dummy_network();

        do_disjktra(&dummy_network, 94, 92, 7);
        do_disjktra(&dummy_network, 91, 94, 4);
        do_disjktra(&dummy_network, 91, 93, 2);
        do_disjktra(&dummy_network, 91, 95, 4);
        do_disjktra(&dummy_network, 92, 95, 6);
        do_disjktra(&dummy_network, 95, 94, 5);
    }

    fn do_disjktra(network: &Network, source: OSMNodeId, destination: OSMNodeId, expected_cost: u64) {
        let maybe_entry = run_dijsktra(source, destination, network, 0, true);

        println!("\ndo_dijkstra network; {:?}", network);

        assert!(maybe_entry.is_some());
        let entry = maybe_entry.unwrap();
        println!("dijkstra result, {:?}", entry);
        assert_eq!(expected_cost, entry.cost);
    }

    fn make_dummy_network() -> Network {
        let network_json = r#"{
        "all_nodes":{
            "91": {"id": 91, "latitude": 0, "longitude": 0},
            "92": {"id": 92, "latitude": 0, "longitude": 0},
            "93": {"id": 93, "latitude": 0, "longitude": 0},
            "94": {"id": 94, "latitude": 0, "longitude": 0},
            "95": {"id": 95, "latitude": 0, "longitude": 0}
        },
        "used_nodes":[],
        "way_info":{},
        "adjacent_arcs":{
            "91": [{"head_node": 94, "distance": 4, "cost": 4, "part_of_way": 1}, {"head_node": 92, "distance": 5, "cost": 5, "part_of_way": 1},{"head_node": 93, "distance": 2, "cost": 2, "part_of_way": 1}],
            "92": [{"head_node": 91, "distance": 5, "cost": 5, "part_of_way": 1}, {"head_node": 93, "distance": 4, "cost": 4, "part_of_way": 1},{"head_node": 95, "distance": 8, "cost": 8, "part_of_way": 1}],
            "93": [{"head_node": 91, "distance": 2, "cost": 2, "part_of_way": 1}, {"head_node": 92, "distance": 4, "cost": 4, "part_of_way": 1},{"head_node": 95, "distance": 2, "cost": 2, "part_of_way": 1},{"head_node": 94, "distance": 3, "cost": 3, "part_of_way": 1}],
            "94": [{"head_node": 91, "distance": 4, "cost": 4, "part_of_way": 1}, {"head_node": 93, "distance": 3, "cost": 3, "part_of_way": 1}],
            "95": [{"head_node": 92, "distance": 8, "cost": 8, "part_of_way": 1}, {"head_node": 93, "distance": 2, "cost": 2, "part_of_way": 1}]
        }
    }
    "#;
        NetworkBuilder::from_json(network_json).unwrap().build_network().unwrap()
    }
}
