// This is free and unencumbered software released into the public domain.

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use std::string::ToString;
use jq::{JsonFilter, JsonFilterError};
use serde_json::Value;

/// Transforms Chromium JSON bookmarks to JSON-LD.
pub struct BookmarksTransform {
    filter: JsonFilter,
}

impl BookmarksTransform {
    pub fn new() -> Result<Self, JsonFilterError> {
        Ok(Self {
            filter: crate::jq::BOOKMARKS.parse()?,
        })
    }

    pub fn execute(&self, input: Value) -> Result<Value, JsonFilterError> {
        self.filter.filter_json(input)
    }

    /// Merges multiple bookmark profiles into a single result
    pub fn execute_multiple(&self, inputs: Vec<Value>) -> Result<Value, JsonFilterError> {
        if inputs.is_empty() {
            return Ok(Value::Object(serde_json::Map::new()));
        }

        if inputs.len() == 1 {
            return self.execute(inputs.into_iter().next().unwrap());
        }

        // Transform each profile individually and collect all items
        let mut all_items = Vec::new();
        
        for input in &inputs {
            // Use BookmarksTransform to convert to JSON-LD
            let result = self.execute(input.clone())?;
            
            // Extract items from the result and add to all_items
            if let Some(items) = result.get("items") {
                if let Some(items_array) = items.as_array() {
                    for item in items_array {
                        all_items.push(item.clone());
                    }
                }
            }
        }
        
        // Create final merged result with all items from all profiles
        let merged_result = serde_json::json!({
            "@context": {
                "know": "https://know.dev/",
                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                "xsd": "http://www.w3.org/2001/XMLSchema#",
                "items": {
                    "@id": "rdfs:member",
                    "@type": "know:UserAccount",
                    "@container": "@set"
                },
                "created": {
                    "@id": "know:created",
                    "@type": "xsd:dateTime"
                },
                "title": {
                    "@id": "know:title",
                    "@language": "en"
                },
                "link": {
                    "@id": "know:link",
                    "@type": "xsd:anyURI"
                }
            },
            "items": all_items
        });
        
        Ok(merged_result)
    }
}
