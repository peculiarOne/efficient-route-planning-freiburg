#[macro_use]
extern crate log;
extern crate env_logger;
extern crate quick_xml;

mod dijkstra;
mod network;
mod utils;

mod osm;

use network::{Network, OSMNodeId};
use osm::load_xml;

use quick_xml::events::Event;
use quick_xml::Reader;
use std::time::Instant;

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

    let oakham_the_avenue: OSMNodeId = 3711862961;
    let oakham_braunston_road: OSMNodeId = 18335097;
    let oakham_tolenthorpe_close: OSMNodeId = 18334319;
    let oakham_woodland_view: OSMNodeId = 18339438;
    let uppingham_queens_road: OSMNodeId = 18327809;
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
    match load_xml::load_network_from_file(file) {
        Ok(network) => network,
        _ => panic!("loading network failed"),
    }
}

#[cfg(test)]
fn from_osm_dummy() -> Option<Network> {
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
    load_xml::load_network_from_string(test_xml)
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

#[cfg(test)]
mod rutland_tests {
    use super::*;

    use std::fs;

    #[test]
    fn sample_node() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();

        let node = rutland_graph.get_node(&488432);
        assert!(node.is_some());
        assert_eq!(52.6555853, node.unwrap().latitude);
        assert_eq!(-0.5134241, node.unwrap().longitude);
    }

    #[test]
    fn total_nodes() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();
        assert_eq!(119638, rutland_graph.node_count());
    }

    #[test]
    fn total_arcs() {
        // TODO can we use a common Network across tests?
        let rutland_graph: Network = from_osm_rutland();
        assert_eq!(42890, rutland_graph.arc_count());
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

        let network = load_xml::load_network_from_string(&xml_string).unwrap();
        // let network = from_osm_rutland();

        const START_NODE: OSMNodeId = 18328098;
        let highways = network.get_node(&START_NODE)
            .adjacent_arcs
            .get(&START_NODE)
            .expect("couldn't find chesnut close");
        println!("chestnut close adjacent arcs: {:?}", highways);
        assert_eq!(3, highways.len());
    }

    #[test]
    fn not_a_highway() {
        const LANDUSE_BOUNDARY_START: OSMNodeId = 19549583;

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

        let network = load_xml::load_network_from_string(&xml_string).unwrap();

        const A_NODE: OSMNodeId = 1917341728;

        println!("network: {}", &network.to_json().unwrap());
        assert_eq!(12, network.arc_count());
        assert_eq!(1, network.adjacent_arcs.get(&A_NODE).unwrap().len());
    }
}
