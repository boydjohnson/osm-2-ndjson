#![feature(fixed_size_array)]

use crate::tags::properties_from_tags;
use clap::{App, Arg, ArgMatches};
use geojson::{feature::Id, Feature, GeoJson, Geometry, Value};
use osmpbfreader::{
    objects::{Node, OsmObj},
    reader::OsmPbfReader,
    NodeId,
};
use std::fs::File;
use std::io::{BufReader, Write};
use std::process::exit;
use store::Store;
use store_impl::Long;

mod store_impl;
mod tags;

fn main() {
    let opts = parse_args();

    let filename = opts.value_of("file").expect("filename is required");

    let tags = opts.value_of("tags");

    let f = match File::open(filename) {
        Ok(f) => f,
        Err(err) => {
            writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
            exit(1);
        }
    };

    let temp_dir = match tempdir::TempDir::new("osm-2-ndjson") {
        Ok(t) => t,
        Err(err) => {
            writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stederr");
            exit(1);
        }
    };

    let mut node_store: Store<Long> = match Store::new(temp_dir.path()) {
        Ok(s) => s,
        Err(e) => {
            writeln!(std::io::stderr(), "{:?}", e).expect("Unable to write to stderr");
            exit(1);
        }
    };

    let mut reader = OsmPbfReader::new(BufReader::new(f));
    for obj in reader.par_iter() {
        match obj {
            Ok(OsmObj::Node(mut node)) => {
                let geometry = Geometry::new(Value::Point(vec![node.lon(), node.lat()]));

                let geojson = GeoJson::Feature(Feature {
                    bbox: None,
                    geometry: Some(geometry),
                    id: Some(Id::Number(node.id.0.into())),
                    properties: Some(properties_from_tags(&mut *node.tags)),
                    foreign_members: None,
                });

                let node_id: NodeId = node.id.into();

                if let Err(err) = node_store.insert(node_id.into(), node) {
                    writeln!(std::io::stderr(), "{:?}", err).expect("Unable to write to stderr");
                }

                if let Err(err) = writeln!(std::io::stdout(), "{}", geojson.to_string()) {
                    writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                }
            }
            Ok(OsmObj::Way(mut way)) => {
                let geometry = if way.is_closed() {
                    Geometry::new(Value::Polygon(vec![]))
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
            Ok(OsmObj::Relation(rel)) => {}
            Err(err) => {
                writeln!(std::io::stderr(), "{:?}", err).expect("Unable to write to stderr");
            }
        }
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("osm-2-ndjson")
        .about("stream osm pbf file to ndjson")
        .arg(
            Arg::with_name("file")
                .help("filename of osm.pbf file")
                .required(true),
        )
        .arg(
            Arg::with_name("tags")
                .short("t")
                .long("tags")
                .help("Filter only tags that match the pattern")
                .required(false),
        )
        .get_matches()
}
