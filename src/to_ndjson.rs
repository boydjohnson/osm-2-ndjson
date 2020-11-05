use crate::{
    store_impl::{Long, OsmIdW},
    tags::{properties_from_tags, TagInner},
};
use geo_types::Polygon;
use geojson::{feature::Id, Feature, GeoJson, Geometry, Value};
use osmpbfreader::{
    objects::{Node, OsmObj},
    reader::OsmPbfReader,
    NodeId, OsmId, StoreObjs,
};
use std::{
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

    let pred = |obj: &OsmObj| {
        (obj.is_node() || obj.is_way())
            && (tags.is_empty()
                || tags
                    .iter()
                    .any(|tags| tags.iter().all(|t| obj.tags().contains(t.key(), t.value()))))
    };

    let mut objects = Objects::default();

    reader.get_objs_and_deps_store(pred, &mut objects).unwrap();

    for (_, obj) in objects.into_iter() {
        match obj {
            OsmObj::Node(mut node) => {
                if !node.tags.is_empty() {
                    let geometry = Geometry::new(Value::Point(vec![node.lon(), node.lat()]));

                    let geojson = GeoJson::Feature(Feature {
                        bbox: None,
                        geometry: Some(geometry),
                        id: Some(Id::Number(node.id.0.into())),
                        properties: Some(properties_from_tags(&mut *node.tags)),
                        foreign_members: None,
                    });

                    if let Err(err) = writeln!(std::io::stdout(), "{}", geojson.to_string()) {
                        writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                    }
                }

                let node_id: NodeId = node.id.into();

                if let Err(err) = node_store.insert(node_id.into(), node) {
                    writeln!(std::io::stderr(), "{:?}", err).expect("Unable to write to stderr");
                }
            }

            OsmObj::Way(mut way) => {
                let geometry = if way.is_closed() {
                    Geometry::new(
                        (&geo_types::Geometry::Polygon(Polygon::new(
                            geo_types::LineString::from(
                                way.nodes
                                    .iter()
                                    .filter_map(|id| {
                                        let node_id: NodeId = (*id).into();
                                        match node_store.get(&node_id.into()) {
                                            Ok(node_opt) => {
                                                if let Some(node) = node_opt {
                                                    let node: Node = node;
                                                    Some((node.lon(), node.lat()))
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
                                        arr.push(item);
                                        arr
                                    }),
                            ),
                            vec![],
                        )))
                            .into(),
                    )
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
            OsmObj::Relation(_) => {}
        }
    }
}

struct Objects(Store<OsmIdW, OsmObj>);

impl Default for Objects {
    fn default() -> Self {
        Objects(Store::tmp_store("osm-objects").unwrap())
    }
}

impl StoreObjs for Objects {
    fn insert(&mut self, key: OsmId, value: OsmObj) {
        let key = OsmIdW(key);

        self.0.insert(key, value).unwrap();
    }

    fn contains_key(&self, key: &OsmId) -> bool {
        let key = OsmIdW(*key);

        self.0.get(&key).unwrap().is_some()
    }
}

impl<'a> IntoIterator for &'a Objects {
    type Item = (OsmIdW, OsmObj);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.iter())
    }
}
