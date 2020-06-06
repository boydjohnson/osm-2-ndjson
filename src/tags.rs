use flat_map::FlatMap;
use serde_json::{to_value, Map, Value};
use std::{collections::HashMap, io::Write};

pub fn properties_from_tags(tags: &mut FlatMap<String, String>) -> Map<String, Value> {
    let mut state = HashMap::new();

    let tags = std::mem::replace(tags, flat_map::FlatMap::new());

    for (key, value) in tags.into_iter() {
        state
            .entry(key)
            .and_modify(|v: &mut Vec<String>| v.push(value.clone()))
            .or_insert(vec![value]);
    }

    let mut properties = Map::new();

    for (key, mut value) in state {
        if value.len() != 1 {
            let v = match to_value(value) {
                Ok(v) => Some(v),
                Err(err) => {
                    writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                    None
                }
            };
            if let Some(v) = v {
                properties.insert(key, v);
            }
        } else {
            let v = match to_value(value.pop()) {
                Ok(v) => Some(v),
                Err(err) => {
                    writeln!(std::io::stderr(), "{}", err).expect("Unable to write to stderr");
                    None
                }
            };
            if let Some(v) = v {
                properties.insert(key, v);
            }
        }
    }
    properties
}

#[derive(Debug, PartialEq)]
pub struct TagInner(String, String);

impl TagInner {
    pub fn key(&self) -> &str {
        &self.0
    }

    pub fn value(&self) -> &str {
        &self.1
    }
}

pub fn split_tags(tags: &str) -> Result<Vec<Vec<TagInner>>, String> {
    let mut result = vec![];

    for item in tags.split(",") {
        let split = item.split("+");

        let mut r = vec![];

        for item in split {
            let mut key_value: Vec<&str> = item.split("=").collect();

            let key = if let Some(key) = key_value.first_mut() {
                Some(std::mem::replace(key, ""))
            } else {
                None
            };

            let value = if let Some(value) = key_value.last_mut() {
                Some(std::mem::replace(value, ""))
            } else {
                None
            };

            match key {
                Some(key) => match value {
                    Some(value) => r.push(TagInner(key.to_string(), value.to_string())),
                    None => return Err(format!("Key {} missing value", key)),
                },
                None => return Err("Missing Key".to_string()),
            }
        }
        result.push(r);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn tags_and_with_or() {
        assert_eq!(
            split_tags("amenity=bar+addr:city=Dresden,amenity=bar+addr:city=Berlin"),
            Ok(vec![
                vec![
                    TagInner("amenity".to_string(), "bar".to_string()),
                    TagInner("addr:city".to_string(), "Dresden".to_string())
                ],
                vec![
                    TagInner("amenity".to_string(), "bar".to_string()),
                    TagInner("addr:city".to_string(), "Berlin".to_string())
                ]
            ])
        );
    }
}
