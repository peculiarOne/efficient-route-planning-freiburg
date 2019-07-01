extern crate quick_xml;

mod dijkstra;
mod network;
mod utils;

use network::{Arc, Network, Node, NodeId};

use failure::Fail;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::error::Error;
use std::fs;
use std::io::ErrorKind;

fn main() {
    println!("Hello, world!");
    let network = from_osm_rutland();
}

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
    let mut in_way = false;
    let mut way_is_highway = false;
    let mut way_is_oneway = false;

    let mut way_nodes = vec![];

    let mut temp_adjacent_arcs = vec![];

    let mut prev_node: Option<NodeId> = None;

    let mut graph = Network::new();

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
                        graph.insert_node(n);
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
                                way_nodes.push(node_id);
                                // if prev_node.is_none() {
                                //     prev_node = Some(node_id);
                                // } else {
                                //     let prev_id = prev_node.unwrap();
                                //     let prev_node = graph.get_node(&prev_id).unwrap();
                                //     let this_node = graph.get_node(&node_id).unwrap();
                                //     let distance = calculate_distance(&prev_node, &this_node);
                                //     let cost = calculate_cost(distance);

                                //     let arc = Arc {
                                //         head_node: node_id,
                                //         distance: distance,
                                //         cost: cost,
                                //     };
                                //     temp_adjacent_arcs.push((prev_id, arc));
                                }
                            }
                            _ => (),
                        };
                    }
                    b"tag" if in_way => {
                        way_is_highway |= is_highway(e);
                        way_is_oneway |= is_oneway(e);
                        },
                    b"node" => {
                        let n = extract_node(e).unwrap();
                        graph.insert_node(n);
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
                        // TODO create arcs here including forward and reverse
                        if way_is_highway {
                            for (k, v) in temp_adjacent_arcs.iter() {
                                graph.insert_arc(*k, v.to_owned());
                            }
                        }
                        prev_node = None;
                        temp_adjacent_arcs.clear();
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

    println!("read network with {} nodes", &graph.nodes.len());
    println!(
        "read network with {} outbound arcs ",
        &graph.adjacent_arcs.len()
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

fn is_oneway(tag: &BytesStart) -> bool {
    let one_way_val = osm_tag_value(tag, "oneway");
    match one_way_val {
        Some(ref val) if val == "true" => true,
        _ => false
    }
}

fn osm_tag_value(tag: &BytesStart, key_to_match: &str) -> Option<String> {
    let osm_key = "k".as_bytes();
    let osm_value = "v".as_bytes();

    let bkey_to_match = key_to_match.as_bytes();
    let mut value = None;

    let mut found_key_to_match = false;
    for attribute in tag.attributes() {
        match attribute {
            Ok(ref attr) if attr.key == osm_key => {
                if attr.unescaped_value().unwrap() == bkey_to_match {
                    found_key_to_match = true;
                }
            }
            Ok(ref attr) if attr.key == osm_value => {
                value = String::from_utf8(attr.unescaped_value().unwrap().iter().map(|u| *u).collect::<Vec<_>>()).ok();
            }
            _ => (),
        }
    }
    if found_key_to_match { value } else { None }
}

fn calculate_distance(a: &Node, b: &Node) -> (f64) {
    0.0
}

fn calculate_cost(distance: f64) -> (u64) {
    0
}

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
    fn total_arcs() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();
        assert_eq!(33477, rutland_graph.total_arcs());
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

        const START_NODE: NodeId = 18328098;
        let highways = rutland_graph
            .adjacent_arcs
            .get(&START_NODE)
            .expect("couldn't find rutland close");
        assert_eq!(3, highways.len());
    }

    #[test]
    fn not_a_highway() {
        const LANDUSE_BOUNDARY_START: NodeId = 19549583;

        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();
        assert!(rutland_graph
            .adjacent_arcs
            .get(&LANDUSE_BOUNDARY_START)
            .is_none());
    }

    #[test]
    fn read_oneway() {
    let file = "data/oneway-way.osm.xml";
    let xml_string = fs::read_to_string(file).expect("couldn't read osm file");

    let network = process_osm_xml(&xml_string);

    const A_NODE: NodeId = 1917341728;

    assert_eq!(12, network.total_arcs());
    println!("network: {}", &network.to_json().unwrap());
    assert_eq!(2, network.adjacent_arcs.get(&A_NODE).unwrap().len());
    }
}
