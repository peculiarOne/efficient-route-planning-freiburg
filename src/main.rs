extern crate quick_xml;

use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::events::attributes::Attribute;
use std::collections::HashMap;
use std::fs;

fn main() {
    println!("Hello, world!");
    read_from_osm_xml();
}

type NodeId = u64;

struct Arc {
    head_node: NodeId,
    cost: u64,
}

struct Network {
    outbound_arcs: HashMap<NodeId, Vec<Arc>>,
}

fn process_way() {

}

fn read_from_osm_xml() -> () {
    let file = "data/rutland-latest.osm.xml";
    let xml_string = fs::read_to_string(file).expect("couldn't read osm file");
    let sample: String = xml_string.chars().into_iter().take(100).collect();
    print!("read from osm file,\n {}", sample);

    let mut reader = Reader::from_str(&xml_string);

    let mut buf = Vec::new();
    let mut way_nodes: Vec<NodeId> = Vec::new();
    let mut in_way = false;
    let mut way_is_highway = false;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"way" => in_way = true,
                    b"nd" if in_way => {
                        // TODO attributes() returns an iterator. need to find the "ref" attribute
                        let first_attr = e.attributes().next().unwrap();
                        let node_ref = match &first_attr {
                            Ok(Attribute { 
                                key: b"ref",
                                value: v }) => Some(v),
                            _ => None,
                        };
                        print!("got attrbute {}", first_attr);
                    },
                    _ => (),
                }
            },
            Ok(Event::End(ref e)) if e.name() == b"way" => {
                if way_is_highway {
                };
                in_way = false;
            },
            Ok(Event::Eof) => break,
            _ => (),
        }
    }
}
