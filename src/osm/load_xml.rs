use failure;
use failure::Fail;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::error;
use std::io::BufRead;
use std::io::ErrorKind;

use crate::network;
use crate::network::{Arc, Network, NetworkBuilder, Node, OSMNodeId, OSMWayId, WayInfo};
use crate::utils;

use crate::osm::constants;

pub fn load_network_from_file(file_path: &str) -> Result<Network, Box<dyn error::Error>> {
    let reader = Reader::from_file(&file_path).map_err(|e| e.compat())?;
    load_network(reader).ok_or(Box::new(std::io::Error::new(
        ErrorKind::Other,
        "failed to load network",
    )))
}

#[cfg(test)]
pub fn load_network_from_string(xml_string: &str) -> Option<Network> {
    let reader = Reader::from_str(xml_string);
    load_network(reader)
}

fn load_network<B: BufRead>(mut reader: Reader<B>) -> Option<Network> {
    let mut buf = Vec::new();
    let mut in_way = false;
    let mut way_is_highway = false;
    let mut way_is_oneway = false;
    let mut way_name = None;
    let mut way_id = 0;

    let mut way_nodes = vec![];

    let mut graph = NetworkBuilder::new();

    println!("enter load_network");

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"way" => {
                    in_way = true;
                    way_id = get_attribute(e, "id").unwrap().parse::<u64>().unwrap();
                    }
                b"node" => {
                    let n = extract_node(e).unwrap();
                    graph.insert_node(n);
                }
                _ => (),
            },
            Ok(Event::Empty(ref e)) => {
                match e.name() {
                    b"nd" if in_way => {
                        // println!("<nd> found in way");
                        // TODO attributes() returns an iterator. need to find the "ref" attribute
                        let node_ref = e.attributes().next().unwrap();
                        let val: Option<OSMNodeId> = match &node_ref {
                            Ok(Attribute {
                                key: b"ref",
                                value: v,
                            }) => {
                                let s = String::from_utf8(v.to_vec()).unwrap();
                                let id: OSMNodeId = s.parse().unwrap();
                                Some(id)
                            }
                            _ => None,
                        };
                        match val {
                            Some(node_id) => way_nodes.push(node_id),
                            _ => (),
                        };
                    }
                    b"tag" if in_way => {
                        way_is_highway |= is_road(e);
                        way_is_oneway |= is_oneway(e);
                        match get_name(e) {
                            Some(name) => way_name = Some(name),
                            _ => (),
                        }
                    }
                    b"node" => {
                        let n = extract_node(e).unwrap();
                        graph.insert_node(n);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"way" => {
                        // TODO create arcs here including forward and reverse
                        if way_is_highway {
                            let arcs = create_arcs(
                                &graph,
                                &way_nodes,
                                way_is_oneway,
                                way_id,
                            );
                            for (k, v) in arcs.iter() {
                                graph.insert_arc(*k, v.to_owned());
                            }
                            graph.insert_way_info(WayInfo { id: way_id, name: way_name })
                        }
                        in_way = false;
                        way_is_highway = false;
                        way_is_oneway = false;
                        way_name = None;
                        way_nodes.clear();
                    }
                    _ => (),
                }
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    println!("about to call build_network");

    println!(
        "read network with {} outbound arcs ",
        &graph.adjacent_arcs.len()
    );

    let network = graph.build_network();
    // if network.is_some() {
    //     println!("read network with {} nodes", &network.unwrap().node_count());
    // }
    network
}

fn extract_node(tag: &BytesStart) -> Result<Node, Box<dyn error::Error>> {
    let mut id = 0;
    let mut lat = 0.0;
    let mut long = 0.0;
    for tag in tag.attributes() {
        match tag.map_err(|e| e.compat())? {
            Attribute {
                key: b"id",
                value: v,
            } => id = utils::bytes_to_string(v)?.parse::<OSMNodeId>()?,
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
    // if id == 0 || lat == 0.0 || long == 0.0 {
    if id == 0 {
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
            latitude: network::degrees_to_i32(lat),
            longitude: network::degrees_to_i32(long),
        })
    }
}

fn create_arcs(
    partial_network: &NetworkBuilder,
    way_nodes: &Vec<OSMNodeId>,
    is_oneway: bool,
    way_id: OSMWayId,
) -> Vec<(OSMNodeId, Arc<OSMNodeId>)> {
    let mut way_iter = way_nodes.iter().peekable();

    let mut arcs = vec![];
    while let Some(node) = way_iter.next() {
        let maybe_next = way_iter.peek();
        match maybe_next {
            Some(next) => {
                let from = partial_network.get_node(node);
                let to = partial_network.get_node(next);
                match (from, to) {
                    (Some(f), Some(t)) => {
                        let dist = calculate_distance(f, t);
                        let cost = calculate_cost(dist);

                        arcs.push((
                            f.id,
                            Arc {
                                head_node: t.id,
                                cost: cost,
                                distance: dist,
                                part_of_way: way_id,
                            },
                        ));

                        if !is_oneway {
                            arcs.push((
                                t.id,
                                Arc {
                                    head_node: f.id,
                                    cost: cost,
                                    distance: dist,
                                    part_of_way: way_id,
                                },
                            ));
                        }
                    }
                    _ => (),
                }
            }
            None => (),
        }
    }
    arcs
}

fn is_road(tag: &BytesStart) -> bool {
    let highway_val = osm_tag_value(tag, "highway");
    match highway_val {
        Some(v) => constants::HIGHWAY_ROAD_TYPES.contains(&v.as_str()),
        None => false,
    }
}

fn is_oneway(tag: &BytesStart) -> bool {
    let one_way_val = osm_tag_value(tag, "oneway");
    match one_way_val {
        Some(ref val) if val == "yes" => true,
        _ => false,
    }
}

fn get_name(tag: &BytesStart) -> Option<String> {
    osm_tag_value(tag, "name")
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
                value = String::from_utf8(
                    attr.unescaped_value()
                        .unwrap()
                        .iter()
                        .map(|u| *u)
                        .collect::<Vec<_>>(),
                )
                .ok();
            }
            _ => (),
        }
    }
    if found_key_to_match {
        value
    } else {
        None
    }
}

fn get_attribute(tag: &BytesStart, key: &str) -> Option<String> {
    for attribute in tag.attributes() {
        match attribute {
            Ok(ref attr) if attr.key == key.as_bytes() => {
                let value = String::from_utf8(
                    attr.unescaped_value()
                        .unwrap()
                        .iter()
                        .map(|u| *u)
                        .collect::<Vec<_>>(),
                )
                .ok();
                return value
            }
            _ => (),
        }
    };
    None
}

fn calculate_distance(a: &Node, b: &Node) -> (u64) {
    utils::haversine_distance_metres(a.lat_long_f64(), b.lat_long_f64())
}

fn calculate_cost(distance: u64) -> (u64) {
    distance
}
