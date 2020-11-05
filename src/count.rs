use crate::tags::TagInner;
use itertools::Itertools;
use osmpbfreader::{OsmObj, OsmPbfReader};
use std::fs::File;
use store::{error::StoreError, key_impl::StringW, Store};

pub fn do_count(osm_file: File, tags: Vec<Vec<TagInner>>) {
    let mut tag_cache = Counter::new();

    let (nodes, open_ways, closed_ways, relations) = OsmPbfReader::new(osm_file).par_iter().fold(
        (0, 0, 0, 0),
        |(mut nodes, mut open_ways, mut closed_ways, mut relations), element| {
            match element {
                Ok(OsmObj::Node(node)) => {
                    if tags
                        .iter()
                        .any(|tags| tags.iter().all(|t| node.tags.contains(t.key(), t.value())))
                        || tags.is_empty()
                    {
                        nodes += 1;

                        for item in node.tags.iter() {
                            tag_cache.insert(format!("{}={}", item.0, item.1)).unwrap();
                        }
                    }
                }
                Ok(OsmObj::Way(way)) => {
                    if tags
                        .iter()
                        .any(|tags| tags.iter().all(|t| way.tags.contains(t.key(), t.value())))
                        || tags.is_empty()
                    {
                        if way.is_open() {
                            open_ways += 1;
                        } else {
                            closed_ways += 1;
                        }
                        for item in way.tags.iter() {
                            tag_cache.insert(format!("{}={}", item.0, item.1)).unwrap();
                        }
                    }
                }
                Ok(OsmObj::Relation(relation)) => {
                    if tags.iter().any(|tags| {
                        tags.iter()
                            .all(|t| relation.tags.contains(t.key(), t.value()))
                    }) || tags.is_empty()
                    {
                        relations += 1;
                        for item in relation.tags.iter() {
                            tag_cache.insert(format!("{}={}", item.0, item.1)).unwrap();
                        }
                    }
                }
                Err(err) => println!("Error: {}", err),
            }
            (nodes, open_ways, closed_ways, relations)
        },
    );
    println!("{} nodes", nodes);
    println!("{} open ways", open_ways);
    println!("{} closed ways", closed_ways);
    println!("{} relations", relations);
    for item in tag_cache.top_n(10) {
        let first = item.0;
        println!("{} {}", first.0, item.1);
    }
}

pub struct Counter {
    items: Store<StringW, u64>,
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            items: Store::tmp_store("osm-count").unwrap(),
        }
    }

    pub fn insert(&mut self, item: String) -> Result<(), StoreError> {
        let item = StringW::from(item);
        self.items
            .entry(item)
            .unwrap()
            .and_modify(|val| *val += 1)
            .unwrap()
            .or_insert(1)?;
        Ok(())
    }

    pub fn top_n(&self, n: usize) -> impl Iterator<Item = (StringW, u64)> {
        Itertools::sorted_by(self.items.iter(), |item1, item2| item1.1.cmp(&item2.1))
            .rev()
            .take(n)
    }
}
