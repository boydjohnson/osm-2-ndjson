use crate::{
    store_impl::Long,
    tags::{properties_from_tags, TagInner},
};
use geojson::{feature::Id, Feature, GeoJson, Geometry, Value};
use osmpbfreader::{
    objects::{Node, OsmObj},
    reader::OsmPbfReader,
    NodeId,
};
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Write},
    process::exit,
};
use store::Store;

pub fn do_to_ndjson(file: File, tags: Vec<Vec<TagInner>>) {
    let temp_dir = match tempdir::TempDir::new("osm-2-ndjson") {
        Ok(t) => t,
        Err(err) => {
            writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stederr");
            exit(1);
        }
    };

    let mut node_store: Store<Long, Node> = match Store::new(temp_dir.path()) {
        Ok(s) => s,
        Err(e) => {
            writeln!(std::io::stderr(), "{:?}", e).expect("Unable to write to stderr");
            exit(1);
        }
    };

    let mut reader = OsmPbfReader::new(BufReader::new(file));
    for obj in reader.par_iter() {
        match obj {
            Ok(OsmObj::Node(mut node)) => {
                if tags.iter().any(|tags| {
                    tags.iter()
                        .all(|t| node.tags.contains(t.key(), t.value()) || node.tags.is_empty())
                }) || tags.is_empty()
                {
                    if node.tags.is_empty() {
                        let node_id: NodeId = node.id.into();

                        if let Err(err) = node_store.insert(node_id.into(), node) {
                            writeln!(std::io::stderr(), "{:?}", err)
                                .expect("Unable to write to stderr");
                        }
                    } else {
                        let geometry = Geometry::new(Value::Point(vec![node.lon(), node.lat()]));

                        let geojson = GeoJson::Feature(Feature {
                            bbox: None,
                            geometry: Some(geometry),
                            id: Some(Id::Number(node.id.0.into())),
                            properties: Some(properties_from_tags(&mut *node.tags)),
                            foreign_members: None,
                        });

                        if let Err(err) = writeln!(std::io::stdout(), "{}", geojson.to_string()) {
                            writeln!(std::io::stderr(), "{}", err)
                                .expect("Unable to write to stderr");
                        }
                    }
                }
            }
            Ok(OsmObj::Way(mut way)) => {
                if tags
                    .iter()
                    .any(|tags| tags.iter().all(|t| way.tags.contains(t.key(), t.value())))
                    || tags.is_empty()
                {
                    let geometry = if way.is_closed() {
                        let mut prior_nodes = VecDeque::new();
                        Geometry::new(Value::Polygon(
                            way.nodes
                                .iter()
                                .filter_map(|id| {
                                    let node_id: NodeId = (*id).into();
                                    match node_store.get(&node_id.into()) {
                                        Ok(node_opt) => {
                                            if let Some(node) = node_opt {
                                                let node: Node = node;
                                                Some(vec![node.lon(), node.lat()])
                                            } else {
                                                writeln!(std::io::stderr(), "Missing Node")
                                                    .expect("Unable to write to stderr");
                                                None
                                            }
                                        }
                                        Err(e) => {
                                            writeln!(std::io::stderr(), "{:?}", e)
                                                .expect("Unable to write to stderr");
                                            None
                                        }
                                    }
                                })
                                .fold(vec![], |mut arr, item| {
                                    if let Some(node) = prior_nodes.pop_front() {
                                        arr.push(vec![node, item.clone()]);
                                    }
                                    prior_nodes.push_back(item);
                                    arr
                                }),
                        ))
                    } else {
                        Geometry::new(Value::LineString(
                            way.nodes
                                .iter()
                                .filter_map(|id| {
                                    let node_id: NodeId = (*id).into();
                                    match node_store.get(&node_id.into()) {
                                        Ok(node_opt) => {
                                            if let Some(node) = node_opt {
                                                let node: Node = node;
                                                Some(vec![node.lon(), node.lat()])
                                            } else {
                                                writeln!(std::io::stderr(), "{}", "Missing node")
                                                    .expect("Unable to write to stderr");
                                                None
                                            }
                                        }
                                        Err(e) => {
                                            writeln!(std::io::stderr(), "{:?}", e)
                                                .expect("Unable to write to stderr");
                                            None
                                        }
                                    }
                                })
                                .collect(),
                        ))
                    };

                    let geojson = GeoJson::Feature(Feature {
                        bbox: None,
                        geometry: Some(geometry),
                        id: Some(Id::Number(way.id.0.into())),
                        properties: Some(properties_from_tags(&mut *way.tags)),
                        foreign_members: None,
                    });

                    if let Err(err) = writeln!(std::io::stdout(), "{}", geojson.to_string()) {
                        writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                    }
                }
            }
            Ok(OsmObj::Relation(rel)) => {}
            Err(err) => {
                writeln!(std::io::stderr(), "{:?}", err).expect("Unable to write to stderr");
            }
        }
    }
}
