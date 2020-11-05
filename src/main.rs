#![feature(array_methods)]

use clap::{App, Arg, ArgMatches, SubCommand};
use std::{fs::File, io::Write, process::exit};
use tags::split_tags;

mod count;
mod store_impl;
mod tags;
mod to_ndjson;

fn main() {
    let opts = parse_args();

    if let Some("count") = opts.subcommand_name() {
        let opts = opts
            .subcommand_matches("count")
            .expect("Tested for subcommand count already");

        let filename = opts.value_of("file").expect("filename is required");

        let tags = if let Some(tags) = opts.value_of("tags") {
            match split_tags(tags) {
                Ok(t) => t,
                Err(e) => {
                    writeln!(std::io::stderr(), "{}", e).expect("Unable to write to stderr");
                    exit(1);
                }
            }
        } else {
            vec![]
        };

        let f = match File::open(filename) {
            Ok(f) => f,
            Err(err) => {
                writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                exit(1);
            }
        };

        count::do_count(f, tags);
    } else if let Some("to-ndjson") = opts.subcommand_name() {
        let opts = opts
            .subcommand_matches("to-ndjson")
            .expect("Tested for subcommand to-ndjson already");

        let filename = opts.value_of("file").expect("filename is required");

        let tags = if let Some(tags) = opts.value_of("tags") {
            match split_tags(tags) {
                Ok(t) => t,
                Err(e) => {
                    writeln!(std::io::stderr(), "{}", e).expect("Unable to write to stderr");
                    exit(1);
                }
            }
        } else {
            vec![]
        };

        let f = match File::open(filename) {
            Ok(f) => f,
            Err(err) => {
                writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                exit(1);
            }
        };

        to_ndjson::do_to_ndjson(f, tags);
    } else {
        writeln!(std::io::stdout(), "{}", opts.usage()).expect("Unable to write to stdout");
    }
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("osm")
        .about("cli tool to inspect osm.pbf files")
        .subcommand(
            SubCommand::with_name("count")
                .about("Displays counts of nodes, ways, relations and related tags")
                .arg(
                    Arg::with_name("file")
                        .help("filename of osm.pbf file")
                        .required(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .short("t")
                        .long("tags")
                        .takes_value(true)
                        .number_of_values(1)
                        .help("Filter only tags that match the pattern")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("to-ndjson")
                .about("Write nodes and ways as Line-Delimited GeoJson to stdout")
                .arg(
                    Arg::with_name("file")
                        .help("filename of osm.pbf file")
                        .required(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .short("t")
                        .long("tags")
                        .takes_value(true)
                        .number_of_values(1)
                        .help("Filter only tags that match the pattern")
                        .required(false),
                ),
        )
        .get_matches()
}
