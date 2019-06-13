extern crate quick_xml;

use quick_xml::events::attributes::{Attribute, Attributes};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::borrow::Cow;
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

fn process_way() {}

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
            Ok(Event::Start(e)) => {
                match e.name() {
                    b"way" => in_way = true,
                    b"nd" if in_way => {
                        // TODO attributes() returns an iterator. need to find the "ref" attribute
                        let node_ref = e.attributes().next().unwrap();
                        let val: Option<NodeId> = match &node_ref {
                            Ok(Attribute {
                                key: b"ref",
                                value: v,
                            }) => {
                                let s = String::from_utf8(v.to_vec()).unwrap();
                                let id:u64 = s.parse().unwrap();
                                Some(id)
                            },
                            _ => None,
                        };
                        // print!("got attrbute {}", node_ref);
                    }
                    b"tag" if in_way => {
                        way_is_highway |= is_highway(e);
                    }
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) if e.name() == b"way" => {
                if way_is_highway {};
                in_way = false;
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
    }
}

fn is_highway(tag: BytesStart) -> bool {
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

fn find_attributes<'a>(attributes: Attributes<'a>, key: &[u8]) -> (Vec<Attribute<'a>>) {
    let f = attributes
        .filter_map(|attr_result| match attr_result {
            Ok(ref attr) if *attr.key == *key => Some(attr_result.unwrap()),
            _ => None,
        })
        .collect::<Vec<_>>();
    // let f = attributes.filter(|a| *a.as_ref().unwrap().key == *key).collect::<Vec<_>>();
    f
}

// fn find_attribute_value<'a>(attributes: Attributes<'a>, key: &[u8]) -> (Option<Result<Cow<'a, [u8]>, quick_xml::Error>>) {
//     let f = attributes.filter_map(|a| {
//         match a.as_ref() {
//             Ok(attr) if *attr.key == *key => Some(a.unwrap().unescaped_value()),
//             _ => None
//         }
//     }).collect::<Vec<_>>();
//     if f.is_empty() { None } else { Some(f[0]) }
// }
// fn find_attribute_value<'a>(attributes: Attributes<'a>, key: &[u8]) -> (Option<Result<Cow<'a, [u8]>, quick_xml::Error>>) {
//     let iFirst = attributes.clone().filter(|a| *(a.unwrap().key) == *key).next();
//     let iFirstVal = iFirst.map(|a| a.unwrap().unescaped_value());
//     // let filtered = attributes.filter(|a| a.unwrap().key == key).map(|a| a.unwrap().unescaped_value()).collect::<Vec<_>>();
//     // let first = filtered.first();
//     iFirstVal
//     // attributes.find(|a| a.unwrap().key == key).map(|a| a.unwrap().unescaped_value())
// }

// TODO read chaper on generics and lifetimes, https://doc.rust-lang.org/stable/book/ch10-00-generics.html
