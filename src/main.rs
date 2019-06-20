extern crate quick_xml;

mod utils;

use failure::Fail;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;

fn main() {
    println!("Hello, world!");
    from_osm_rutland();
}

type NodeId = u64;

struct Node {
    id: NodeId,
    latitude: f64,
    longitude: f64,
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

#[derive(Clone)]
#[derive(Debug)]
struct Arc {
    head_node: NodeId,
    distance: f64,
    cost: u64,
}

struct Network {
    nodes: HashMap<NodeId, Node>,
    highways_map: HashMap<NodeId, Vec<Vec<Arc>>>, // possibly mltiple arcs with the same start
}
impl Network {
    fn from_nodes(nodes: HashMap<NodeId, Node>) -> Network {
        Network {
            nodes: nodes,
            highways_map: HashMap::new(),
        }
    }

    fn insert_arcs(&mut self, start_node: NodeId, arcs: Vec<Arc>) {
        let existing = self.highways_map.entry(start_node).or_insert(vec![]);
        existing.push(arcs);
    }

    fn total_highways(&self) -> usize {
        self.highways_map.values().map(|v| v.len()).sum()
    }
}

fn process_way() {}

fn from_osm_rutland() -> Network {

    let file = "data/rutland-latest.osm.xml";
    let xml_string = fs::read_to_string(file).expect("couldn't read osm file");

    // echo_xml(&xml_string);
    process_osm_xml(&xml_string)
}

fn from_osm_dummy() -> Network {
    let test_xml = r#"<osm>
	<node>
		<tag k="odbl" v="clean"/>
	</node>
        <way>
            <nd>
            </nd>
        </way>
    </osm>"#;

    // echo_xml(&test_xml);
    process_osm_xml(test_xml)
}

fn _echo_xml(xml_string: &str) -> () {
    let mut reader = Reader::from_str(&xml_string);

    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                println!("<{}>", String::from_utf8(e.name().to_vec()).unwrap())
            }
            Ok(Event::Empty(ref e)) => {
                println!("<{}/>", String::from_utf8(e.name().to_vec()).unwrap())
            }
            Ok(Event::End(ref e)) => {
                println!("</{}>", String::from_utf8(e.name().to_vec()).unwrap())
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
}

fn process_osm_xml(xml_string: &str) -> Network {
    let mut reader = Reader::from_str(&xml_string);

    let mut buf = Vec::new();
    let mut all_nodes: HashMap<NodeId, Node> = HashMap::new();
    let mut in_way = false;
    let mut way_is_highway = false;

    let mut network_arcs: Vec<(NodeId, Vec<Arc>)> = vec![];
    let mut way_first_node: Option<NodeId> = None;
    let mut way_arcs: Vec<Arc> = vec![];

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // println!(
                //     "start of tag {}, in_way={}",
                //     String::from_utf8(e.name().to_vec()).unwrap(),
                //     in_way
                // );
                match e.name() {
                    b"way" => {
                        // println!("<way> found");
                        in_way = true
                    }
                    b"node" => {
                        let n = extract_node(e).unwrap();
                        all_nodes.insert(n.id, n);
                    }
                    _ => (),
                }
            }
            Ok(Event::Empty(ref e)) => {
                match e.name() {
                    b"nd" if in_way => {
                        // println!("<nd> found in way");
                        // TODO attributes() returns an iterator. need to find the "ref" attribute
                        let node_ref = e.attributes().next().unwrap();
                        let val: Option<NodeId> = match &node_ref {
                            Ok(Attribute {
                                key: b"ref",
                                value: v,
                            }) => {
                                let s = String::from_utf8(v.to_vec()).unwrap();
                                let id: u64 = s.parse().unwrap();
                                Some(id)
                            }
                            _ => None,
                        };
                        match val {
                            Some(node_id) => {
                                if way_first_node.is_none() {
                                    way_first_node = Some(node_id);
                                } else {
                                    let prev_id = if way_arcs.is_empty() {
                                        way_first_node.unwrap()
                                    } else {
                                        way_arcs.last().unwrap().head_node
                                    };
                                    let prev_node = all_nodes.get(&prev_id).unwrap();
                                    let this_node = all_nodes.get(&node_id).unwrap();
                                    let distance = calculate_distance(&prev_node, &this_node);
                                    let cost = calculate_cost(distance);

                                    let arc = Arc {
                                        head_node: node_id,
                                        distance: distance,
                                        cost: cost,
                                    };
                                    way_arcs.push(arc);
                                }
                            }
                            _ => (),
                        };
                    }
                    b"tag" if in_way => way_is_highway |= is_highway(e),
                    b"node" => {
                        let n = extract_node(e).unwrap();
                        all_nodes.insert(n.id, n);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) => {
                // println!(
                //     "start of tag {}",
                //     String::from_utf8(e.name().to_vec()).unwrap()
                // );
                match e.name() {
                    b"way" => {
                        if way_is_highway {
                            match way_first_node {
                                Some(id) => network_arcs.push((id, way_arcs.to_vec())),
                                _ => {}
                            };
                        };
                        way_first_node = None;
                        way_arcs.clear();
                        in_way = false;
                        way_is_highway = false;
                    }
                    _ => (),
                }
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }

    let mut graph = Network::from_nodes(all_nodes);
    for tup in network_arcs {
        graph.insert_arcs(tup.0, tup.1);
    }

    println!("read network with {} nodes", &graph.nodes.len());
    println!(
        "read network with {} outbound arcs ",
        &graph.highways_map.len()
    );

    graph
}

fn extract_node(tag: &BytesStart) -> Result<Node, Box<dyn Error>> {
    let mut id = 0;
    let mut lat = 0.0;
    let mut long = 0.0;
    for tag in tag.attributes() {
        match tag.map_err(|e| e.compat())? {
            Attribute {
                key: b"id",
                value: v,
            } => id = utils::bytes_to_string(v)?.parse::<NodeId>()?,
            Attribute {
                key: b"lat",
                value: v,
            } => lat = utils::bytes_to_string(v)?.parse::<f64>()?,
            Attribute {
                key: b"lon",
                value: v,
            } => long = utils::bytes_to_string(v)?.parse::<f64>()?,
            _ => (),
        }
    }
    if id == 0 || lat == 0.0 || long == 0.0 {
        println!(
            "problem extracting node id {}, lat {}, long {}",
            id, lat, long
        );
        Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            "invalid node",
        )))
    } else {
        Ok(Node {
            id: id,
            latitude: lat,
            longitude: long,
        })
    }
}

fn is_highway(tag: &BytesStart) -> bool {
    let osm_key = "k".as_bytes();
    let osm_value = "v".as_bytes();

    let osm_highway = "highway".as_bytes();

    let mut found_highway_tag = false;
    for attribute in tag.attributes() {
        match attribute {
            Ok(ref attr) if attr.key == osm_key => {
                if attr.unescaped_value().unwrap() == osm_highway {
                    found_highway_tag = true;
                }
            }
            _ => (),
        }
    }
    found_highway_tag
}

fn calculate_distance(a: &Node, b: &Node) -> (f64) {
    0.0
}

fn calculate_cost(distance: f64) -> (u64) {
    0
}

// TODO read chaper on generics and lifetimes, https://doc.rust-lang.org/stable/book/ch10-00-generics.html

#[cfg(test)]
mod rutland_tests {
    use super::*;

    #[test]
    fn sample_node() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();

        let node = rutland_graph.nodes.get(&488432);
        assert!(node.is_some());
        assert_eq!(52.6555853, node.unwrap().latitude);
        assert_eq!(-0.5134241, node.unwrap().longitude);
    }

#[test]
    fn total_nodes() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();
        assert_eq!(119638, rutland_graph.nodes.len());
    }

    #[test]
    fn total_ways() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();
        assert_eq!(4667, rutland_graph.total_highways());

    }

	// <way id="3753821" version="1" timestamp="2006-10-20T13:33:58Z" changeset="0">
	// 	<nd ref="18328098"/>
	// 	<nd ref="18328116"/>
	// 	<nd ref="18328115"/>
	// 	<nd ref="18328114"/>
	// 	<tag k="name" v="Chestnut Close"/>
	// 	<tag k="highway" v="residential"/>
	// 	<tag k="created_by" v="JOSM"/>
	// </way>
    #[test]
    fn chestnut_close() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();

        const START_NODE:NodeId = 18328098;
        let highways = rutland_graph.highways_map.get(&START_NODE).expect("couldn't find rutland close");
        assert!(highways.len() == 1);

        let highway = highways.iter().next().unwrap();
        assert_eq!(3, highway.len());
    }

    #[test]
    fn not_a_highway() {
        const LANDUSE_BOUNDARY_START: NodeId = 19549583;

        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();
        assert!(rutland_graph.highways_map.get(&LANDUSE_BOUNDARY_START).is_none());
    }
}