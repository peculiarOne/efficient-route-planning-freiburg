extern crate quick_xml;

mod utils;

use failure::Fail;
use quick_xml::events::attributes::{Attribute, Attributes};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::fs;
use std::error::Error;
use std::hash::{Hash, Hasher};

fn main() {
    println!("Hello, world!");
    from_osm_xml();
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

struct Arc {
    head_node: NodeId,
    cost: u64,
}

struct Network {
    outbound_arcs: HashMap<NodeId, Vec<Arc>>,
}

fn process_way() {}

fn from_osm_xml() -> () {
    let test_xml = r#"<osm>
	<node>
		<tag k="odbl" v="clean"/>
	</node>
        <way>
            <nd>
            </nd>
        </way>
    </osm>"#;

    let file = "data/rutland-latest.osm.xml";
    let xml_string = fs::read_to_string(file).expect("couldn't read osm file");
    let sample: String = xml_string.chars().into_iter().take(100).collect();
    // print!("read from osm file,\n {}", sample);

    // echo_xml(&xml_string);
    // echo_xml(&test_xml);
    process_osm_xml(&xml_string);
}

fn echo_xml(xml_string: &str) -> () {
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

fn process_osm_xml(xml_string: &str) -> () {
    let mut reader = Reader::from_str(&xml_string);

    let mut buf = Vec::new();
    let mut all_nodes: HashSet<Node> = HashSet::new();
    let mut way_nodes: HashSet<NodeId> = HashSet::new();
    let mut in_way = false;
    let mut way_is_highway = false;
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
                        println!("<way> found");
                        in_way = true
                    }
                    b"node" => {

                    }
                    _ => (),
                }
            }
            Ok(Event::Empty(ref e)) => {
                match e.name() {
                    b"nd" if in_way => {
                        println!("<nd> found in way");
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
                            Some(node_id) => way_nodes.insert(node_id),
                            _ => false,
                        };
                    }
                    b"tag" if in_way => {
                        println!("<tag> found in way");
                        way_is_highway |= is_highway(e);
                    }
                    b"node" => { all_nodes.insert(extract_node(e).unwrap()); },
                    _ => (),
                }
            }
            Ok(Event::End(ref e)) => {
                println!(
                    "start of tag {}",
                    String::from_utf8(e.name().to_vec()).unwrap()
                );
                match e.name() {
                    b"way" => {
                        if way_is_highway {};
                        in_way = false;
                    }
                    _ => (),
                }
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }

    println!("read {} highway nodes from network", way_nodes.len());
    println!("create {} Nodes", all_nodes.len());
}

fn extract_node(tag: &BytesStart) -> Result<Node, Box<dyn Error>> {
    let mut id = 0;
    let mut lat = 0.0;
    let mut long = 0.0;
    for tag in tag.attributes() {
        match tag.map_err(|e| e.compat())? {
            Attribute{key: b"id", value: v} => id = utils::bytes_to_string(v)?.parse::<NodeId>()?,
            Attribute{key: b"lat", value: v} => lat = utils::bytes_to_string(v)?.parse::<f64>()?,
            Attribute{key: b"long", value: v} => long = utils::bytes_to_string(v)?.parse::<f64>()?,
            _ => (),
        }
    }
    if id == 0 || lat == 0.0 || long == 0.0 {
        Err(Box::new(std::io::Error::new(ErrorKind::Other, "invalid node")))
    } else {
        Ok(Node { id: id, latitude: lat, longitude: long })
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

// TODO read chaper on generics and lifetimes, https://doc.rust-lang.org/stable/book/ch10-00-generics.html
