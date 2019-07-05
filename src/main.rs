#[macro_use]
extern crate log;
extern crate env_logger;
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
use std::io::BufRead;
use std::io::ErrorKind;
use std::time::{Duration, Instant};

// const OSM_DATA_FILE: &str = "data/rutland-latest.osm.xml";
const OSM_DATA_FILE: &str = "/home/wayne/Downloads/great-britain-latest.osm.xml";

fn main() {
    env_logger::init();

    println!("Hello, world!");
    let start_load_network = Instant::now();
    let network = from_osm_file(OSM_DATA_FILE);
    let duration_load_network = start_load_network.elapsed();
    println!(
        "time to load network {} {:?}",
        OSM_DATA_FILE, duration_load_network
    );

    let oakham_the_avenue: NodeId = 3711862961;
    let oakham_braunston_road: NodeId = 18335097;
    let oakham_tolenthorpe_close: NodeId = 18334319;
    let oakham_woodland_view: NodeId = 18339438;
    let uppingham_queens_road: NodeId = 18327809;
    let result = dijkstra::run_dijsktra(
        oakham_braunston_road,
        uppingham_queens_road,
        &network,
        15000,
        true,
    );
    match result {
        Some(entry) => {
            println!("path result cost: {}", entry.cost);
            println!("ways travelled: {}", entry.report_traversed_ways());
        }
        None => println!("no path found!!"),
    }

    let start = Instant::now();
    let whole_network_result = dijkstra::run_dijsktra(oakham_the_avenue, 0, &network, 0, false);
    let duration = start.elapsed();
    println!("time to complete full dijkstra {:?}", duration);
}

fn from_osm_rutland() -> Network {
    from_osm_file("data/rutland-latest.osm.xml")
}

fn from_osm_file(file: &str) -> Network {
    // let xml_string = fs::read_to_string(file).expect("couldn't read osm file");

    println!("process {}", file);
    // echo_xml(&xml_string);
    match load_network_from_file(file) {
        Ok(network) => network,
        _ => panic!("loading network failed"),
    }
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
    load_network_from_string(test_xml)
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

fn load_network_from_file(file_path: &str) -> Result<Network, quick_xml::Error> {
    let mut reader = Reader::from_file(&file_path)?;
    Ok(load_network(reader))
}

fn load_network_from_string(xml_string: &str) -> Network {
    let mut reader = Reader::from_str(xml_string);
    load_network(reader)
}

fn load_network<B: BufRead>(mut reader: Reader<B>) -> Network {
    let mut buf = Vec::new();
    let mut in_way = false;
    let mut way_is_highway = false;
    let mut way_is_oneway = false;
    let mut way_name = None;

    let mut way_nodes = vec![];

    let mut graph = Network::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"way" => in_way = true,
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
                                way_name.as_ref().map(|n| n.as_str()),
                            );
                            for (k, v) in arcs.iter() {
                                graph.insert_arc(*k, v.to_owned());
                            }
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

    println!("read network with {} nodes", &graph.nodes.len());
    println!(
        "read network with {} outbound arcs ",
        &graph.adjacent_arcs.len()
    );

    graph
}

fn create_arcs(
    partial_network: &Network,
    way_nodes: &Vec<NodeId>,
    is_oneway: bool,
    way_name: Option<&str>,
) -> Vec<(NodeId, Arc)> {
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
                                part_of_way: way_name.map(|n| n.into()),
                            },
                        ));

                        if !is_oneway {
                            arcs.push((
                                t.id,
                                Arc {
                                    head_node: f.id,
                                    cost: cost,
                                    distance: dist,
                                    part_of_way: way_name.map(|n| n.into()),
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
            latitude: lat,
            longitude: long,
        })
    }
}

const HIGHWAY_ROAD_TYPES: [&'static str; 13] = [
    "motorway",
    "trunk",
    "primary",
    "secondary",
    "tertiary",
    "unclassified",
    "residential",
    "motorway_link",
    "trunk_link",
    "primary_link",
    "secondary_link",
    "tertiary_link",
    "service",
];

fn is_road(tag: &BytesStart) -> bool {
    let highway_val = osm_tag_value(tag, "highway");
    match highway_val {
        Some(v) => HIGHWAY_ROAD_TYPES.contains(&v.as_str()),
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

fn calculate_distance(a: &Node, b: &Node) -> (u64) {
    let from_lat_long = (a.latitude, a.longitude);
    let to_lat_long = (b.latitude, b.longitude);
    utils::haversine_distance_metres(from_lat_long, to_lat_long)
}

fn calculate_cost(distance: u64) -> (u64) {
    distance
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
        assert_eq!(42890, rutland_graph.total_arcs());
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
        let file = "data/rutland-tiny.osm.xml";
        let xml_string = fs::read_to_string(file).expect("couldn't read osm file");

        let network = load_network_from_string(&xml_string);
        // let network = from_osm_rutland();

        const START_NODE: NodeId = 18328098;
        let highways = network
            .adjacent_arcs
            .get(&START_NODE)
            .expect("couldn't find chesnut close");
        println!("chestnut close adjacent arcs: {:?}", highways);
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

        let network = load_network_from_string(&xml_string);

        const A_NODE: NodeId = 1917341728;

        println!("network: {}", &network.to_json().unwrap());
        assert_eq!(12, network.total_arcs());
        assert_eq!(1, network.adjacent_arcs.get(&A_NODE).unwrap().len());
    }
}
