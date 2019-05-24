extern crate quick_xml;

use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::events::attributes::{Attribute, Attributes};
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
                        let node_ref = e.attributes().next().unwrap();
                        match &node_ref {
                            Ok(Attribute { 
                                key: b"ref",
                                value: v }) => Some(v),
                            _ => None,
                        };
                        print!("got attrbute {}", node_ref);
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

// TODO read chaper on generics and lifetimes, https://doc.rust-lang.org/stable/book/ch10-00-generics.html
fn find_attribute(attributes: Attributes, key: &[u8]) -> (Option<Result<Attribute, quick_xml::Error>>) {
    let found = attributes.find(|att| {
        let ua = att.unwrap();
        ua.key == key
    });
    found
}
