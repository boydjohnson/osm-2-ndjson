use crate::tags::properties_from_tags;
use clap::{App, Arg, ArgMatches};
use geojson::{feature::Id, Feature, GeoJson, Geometry, Value};
use osmpbfreader::{objects::OsmObj, reader::OsmPbfReader};
use std::fs::File;
use std::io::{BufReader, Write};
use std::process::exit;

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

                if let Err(err) = writeln!(std::io::stdout(), "{}", geojson.to_string()) {
                    writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                }
            }
            Ok(OsmObj::Way(way)) => {}
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
