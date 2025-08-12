// This is free and unencumbered software released into the public domain.

use miette::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::format;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

/// Bookmark extracted from Arc's StorableSidebar.json
#[derive(Debug)]
pub struct ArcBookmark {
    pub title: String,
    pub url: String,
    pub created_at: Option<f64>,
}

fn discover_pinned_container_id(arc_data: &Value) -> Option<String> {
    let mut parent_id_counts: HashMap<String, usize> = HashMap::new();

    if let Some(sidebar) = arc_data.get("sidebar") {
        if let Some(containers) = sidebar.get("containers") {
            if let Some(containers_array) = containers.as_array() {
                for container in containers_array {
                    if let Some(items) = container.get("items") {
                        if let Some(items_array) = items.as_array() {
                            for item in items_array {
                                if let Some(parent_id) =
                                    item.get("parentID").and_then(|pid| pid.as_str())
                                {
                                    if parent_id != "null" {
                                        if let Some(data) = item.get("data") {
                                            if let Some(tab) = data.get("tab") {
                                                let has_saved_url = tab
                                                    .get("savedURL")
                                                    .and_then(|u| u.as_str())
                                                    .map(|u| !u.is_empty())
                                                    .unwrap_or(false);
                                                let has_saved_title = tab
                                                    .get("savedTitle")
                                                    .and_then(|t| t.as_str())
                                                    .map(|t| !t.is_empty() && t != "null")
                                                    .unwrap_or(false);

                                                if has_saved_url && has_saved_title {
                                                    *parent_id_counts
                                                        .entry(parent_id.to_string())
                                                        .or_insert(0) += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    parent_id_counts
        .into_iter()
        .fold(None, |acc, (id, count)| match acc {
            Some((acc_id, acc_count)) => {
                if count > acc_count || (count == acc_count && id < acc_id) {
                    Some((id, count))
                } else {
                    Some((acc_id, acc_count))
                }
            },
            None => Some((id, count)),
        })
        .map(|(id, _)| id)
}

fn discover_profile_containers(arc_data: &Value) -> HashMap<String, String> {
    let mut profile_containers: HashMap<String, String> = HashMap::new();

    if let Some(sidebar) = arc_data.get("sidebar") {
        if let Some(containers) = sidebar.get("containers") {
            if let Some(containers_array) = containers.as_array() {
                for container in containers_array {
                    if let Some(items) = container.get("items") {
                        if let Some(items_array) = items.as_array() {
                            for item in items_array {
                                if let Some(data) = item.get("data") {
                                    if let Some(item_container) = data.get("itemContainer") {
                                        if let Some(container_type) =
                                            item_container.get("containerType")
                                        {
                                            if let Some(top_apps) = container_type.get("topApps") {
                                                if let Some(top_apps_data) = top_apps.get("_0") {
                                                    if let Some(custom) =
                                                        top_apps_data.get("custom")
                                                    {
                                                        if let Some(custom_data) = custom.get("_0")
                                                        {
                                                            if let Some(directory_basename) =
                                                                custom_data.get("directoryBasename")
                                                            {
                                                                if let Some(profile_name) =
                                                                    directory_basename.as_str()
                                                                {
                                                                    if let Some(id) = item
                                                                        .get("id")
                                                                        .and_then(|i| i.as_str())
                                                                    {
                                                                        profile_containers.insert(
                                                                            profile_name
                                                                                .to_string(),
                                                                            id.to_string(),
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    if top_apps_data.get("default").is_some() {
                                                        if let Some(id) =
                                                            item.get("id").and_then(|i| i.as_str())
                                                        {
                                                            profile_containers.insert(
                                                                "Default".to_string(),
                                                                id.to_string(),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    profile_containers
}

fn map_numeric_profile_to_name(arc_data: &Value, profile_index: u32) -> Option<String> {
    let profile_containers = discover_profile_containers(arc_data);

    if profile_containers.is_empty() {
        return None;
    }

    // Sort profile names to ensure consistent ordering
    let mut sorted_profiles: Vec<_> = profile_containers.keys().collect();
    sorted_profiles.sort();

    // Convert 1-based index to 0-based array index
    let array_index = (profile_index as usize).saturating_sub(1);

    if array_index < sorted_profiles.len() {
        Some(sorted_profiles[array_index].clone())
    } else {
        None
    }
}

pub fn extract_arc_bookmarks_for_profile(
    arc_data: &Value,
    target_profile: Option<&str>,
) -> Result<Vec<ArcBookmark>> {
    if target_profile.is_some() {
        return extract_arc_bookmarks_from_sidebar(arc_data, target_profile);
    }

    // Try new JSON-LD format first
    if let Some(items) = arc_data.get("items") {
        if let Some(items_array) = items.as_array() {
            let mut bookmarks = Vec::new();
            for item in items_array {
                if let Some(bookmark) = extract_bookmark_from_jsonld_item(item) {
                    bookmarks.push(bookmark);
                }
            }
            return Ok(bookmarks);
        }
    }

    extract_arc_bookmarks_from_sidebar(arc_data, target_profile)
}

pub fn extract_arc_bookmarks_for_numeric_profile(
    arc_data: &Value,
    profile_index: Option<u32>,
) -> Result<Vec<ArcBookmark>> {
    match profile_index {
        Some(0) | None => {
            // Return all bookmarks from all profiles
            extract_arc_bookmarks_for_profile(arc_data, None)
        },
        Some(index) => {
            // Map numeric index to profile name
            if let Some(profile_name) = map_numeric_profile_to_name(arc_data, index) {
                extract_arc_bookmarks_for_profile(arc_data, Some(&profile_name))
            } else {
                Err(miette::miette!(
                    "Profile index {} not found. Available profiles: {}",
                    index,
                    discover_profile_containers(arc_data)
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        },
    }
}

fn extract_arc_bookmarks_from_sidebar(
    arc_data: &Value,
    target_profile: Option<&str>,
) -> Result<Vec<ArcBookmark>> {
    let mut bookmarks = Vec::new();
    let profile_containers = discover_profile_containers(arc_data);

    if profile_containers.is_empty() {
        // Fallback to old logic
        let pinned_container_id = discover_pinned_container_id(arc_data).ok_or_else(|| {
            miette::miette!("Could not find any profile containers or pinned bookmarks in Arc data")
        })?;

        if let Some(sidebar) = arc_data.get("sidebar") {
            if let Some(containers) = sidebar.get("containers") {
                if let Some(containers_array) = containers.as_array() {
                    for container in containers_array {
                        if let Some(items) = container.get("items") {
                            if let Some(items_array) = items.as_array() {
                                for item in items_array {
                                    if let Some(bookmark) =
                                        extract_bookmark_from_item(item, &pinned_container_id)
                                    {
                                        bookmarks.push(bookmark);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        if let Some(target_profile_name) = target_profile {
            if let Some(container_id) = profile_containers.get(target_profile_name) {
                if let Some(sidebar) = arc_data.get("sidebar") {
                    if let Some(containers) = sidebar.get("containers") {
                        if let Some(containers_array) = containers.as_array() {
                            for container in containers_array {
                                if let Some(items) = container.get("items") {
                                    if let Some(items_array) = items.as_array() {
                                        for item in items_array {
                                            if let Some(parent_id) =
                                                item.get("parentID").and_then(|pid| pid.as_str())
                                            {
                                                if parent_id == container_id {
                                                    if let Some(bookmark) =
                                                        extract_bookmark_from_item_with_profile(
                                                            item,
                                                            container_id,
                                                            target_profile_name,
                                                        )
                                                    {
                                                        bookmarks.push(bookmark);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                return Err(miette::miette!(
                    "Profile '{}' not found. Available profiles: {}",
                    target_profile_name,
                    profile_containers
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        } else {
            // Extract from all profiles
            for (profile_name, container_id) in profile_containers {
                if let Some(sidebar) = arc_data.get("sidebar") {
                    if let Some(containers) = sidebar.get("containers") {
                        if let Some(containers_array) = containers.as_array() {
                            for container in containers_array {
                                if let Some(items) = container.get("items") {
                                    if let Some(items_array) = items.as_array() {
                                        for item in items_array {
                                            if let Some(parent_id) =
                                                item.get("parentID").and_then(|pid| pid.as_str())
                                            {
                                                if parent_id == container_id {
                                                    if let Some(bookmark) =
                                                        extract_bookmark_from_item_with_profile(
                                                            item,
                                                            &container_id,
                                                            &profile_name,
                                                        )
                                                    {
                                                        bookmarks.push(bookmark);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(bookmarks)
}

pub fn extract_arc_bookmarks(arc_data: &Value) -> Result<Vec<ArcBookmark>> {
    extract_arc_bookmarks_for_profile(arc_data, None)
}

fn extract_bookmark_from_jsonld_item(item: &Value) -> Option<ArcBookmark> {
    let item_type = item.get("@type")?.as_str()?;
    if item_type != "know:Bookmark" {
        return None;
    }

    let title = item.get("title")?.as_str()?.to_string();
    let url = item.get("link")?.as_str()?.to_string();
    let created_at = None;

    Some(ArcBookmark {
        title,
        url,
        created_at,
    })
}

fn extract_bookmark_from_item(item: &Value, pinned_container_id: &str) -> Option<ArcBookmark> {
    let parent_id = item.get("parentID")?.as_str()?;
    if parent_id != pinned_container_id {
        return None;
    }

    let data = item.get("data")?;
    let tab = data.get("tab")?;

    let saved_url = tab.get("savedURL")?.as_str()?.to_string();
    let saved_title = tab.get("savedTitle")?.as_str()?.to_string();
    let created_at = item.get("createdAt").and_then(|c| c.as_f64());

    Some(ArcBookmark {
        title: saved_title,
        url: saved_url,
        created_at,
    })
}

fn extract_bookmark_from_item_with_profile(
    item: &Value,
    _container_id: &str,
    _profile_name: &str,
) -> Option<ArcBookmark> {
    let data = item.get("data")?;
    let tab = data.get("tab")?;

    let saved_url = tab.get("savedURL")?.as_str()?.to_string();
    let saved_title = tab.get("savedTitle")?.as_str()?.to_string();
    let created_at = item.get("createdAt").and_then(|c| c.as_f64());

    Some(ArcBookmark {
        title: saved_title,
        url: saved_url,
        created_at,
    })
}

pub fn convert_arc_bookmarks_to_chromium(arc_data: Value, profile: Option<&str>) -> Result<Value> {
    let bookmarks = extract_arc_bookmarks_for_profile(&arc_data, profile)?;
    let mut counter = 0;
    let mut chromium_bookmarks = Vec::new();

    for bookmark in bookmarks {
        counter += 1;

        let date_added = if let Some(created_at) = bookmark.created_at {
            convert_cf_absolute_time(created_at)
        } else {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            ((now.as_secs() + 11644473600) * 1000000) as i64
        };

        let chromium_bookmark = serde_json::json!({
            "guid": format!("arc-{}-{}", date_added, counter),
            "name": bookmark.title,
            "url": bookmark.url,
            "type": "url",
            "date_added": date_added,
        });

        chromium_bookmarks.push(chromium_bookmark);
    }

    let result = serde_json::json!({
        "roots": {
            "bookmark_bar": {
                "children": chromium_bookmarks,
                "type": "folder"
            },
            "other": {
                "children": [],
                "type": "folder"
            }
        }
    });

    Ok(result)
}

pub fn convert_arc_bookmarks_to_chromium_numeric(
    arc_data: Value,
    profile_index: Option<u32>,
) -> Result<Value> {
    let bookmarks = extract_arc_bookmarks_for_numeric_profile(&arc_data, profile_index)?;
    let mut counter = 0;
    let mut chromium_bookmarks = Vec::new();

    for bookmark in bookmarks {
        counter += 1;

        let date_added = if let Some(created_at) = bookmark.created_at {
            convert_cf_absolute_time(created_at)
        } else {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            ((now.as_secs() + 11644473600) * 1000000) as i64
        };

        let chromium_bookmark = serde_json::json!({
            "guid": format!("arc-{}-{}", date_added, counter),
            "name": bookmark.title,
            "url": bookmark.url,
            "type": "url",
            "date_added": date_added,
        });

        chromium_bookmarks.push(chromium_bookmark);
    }

    let result = serde_json::json!({
        "roots": {
            "bookmark_bar": {
                "children": chromium_bookmarks,
                "type": "folder"
            },
            "other": {
                "children": [],
                "type": "folder"
            }
        }
    });

    Ok(result)
}

pub fn convert_cf_absolute_time(cf_absolute_time: f64) -> i64 {
    let seconds_between_1970_and_2001 = 978307200.0;
    let unix_timestamp_seconds = cf_absolute_time + seconds_between_1970_and_2001;
    ((unix_timestamp_seconds + 11644473600.0) * 1000000.0) as i64
}
