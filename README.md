# A simple cli tool for turning osm.pbf files into ndjson

### usage

```
osm-2-ndjson 0.1.0
cli tool to turn osm.pbf files into ndjson

USAGE:
    osm-2-ndjson [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    count        Displays counts of nodes, ways, relations and related tags
    help         Prints this message or the help of the given subcommand(s)
    to-ndjson    Write nodes and ways as ndjson GeoJson to stdout
```

#### osm-2-ndjson to-ndjson
```
osm-2-ndjson-to-ndjson 
Write nodes and ways as ndjson GeoJson to stdout

USAGE:
    osm-2-ndjson to-ndjson [OPTIONS] <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --tags <tags>    Filter only tags that match. '+' is 'and' and ',' is or

ARGS:
    <file>    filename of osm.pbf file
```

#### osm-2-ndjson count
```
osm-2-ndjson-count 
Displays counts of nodes, ways, relations and related tags

USAGE:
    osm-2-ndjson count [OPTIONS] <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --tags <tags>    Filter only tags that match. '+' is 'and' and ',' is or

ARGS:
    <file>    filename of osm.pbf file
```

### Installing

Right now 

```
git clone git@github.com/boydjohnson/osm-2-ndjson.git
cd osm-2-ndjson
cargo build --release
```

Soon there will be debian packages
