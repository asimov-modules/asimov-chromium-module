/// Maps a 1-based profile index to the corresponding profile name
///
/// Profile indices are mapped to sorted profile names for consistent ordering.
/// Index 1 maps to the first profile alphabetically, index 2 to the second, etc.
pub fn map_numeric_profile_to_name(
    available_profiles: &[std::string::String],
    profile_index: u32,
) -> Option<std::string::String> {
    if profile_index == 0 || profile_index > available_profiles.len() as u32 {
        return None;
    }

    let mut sorted_profiles = available_profiles.to_vec();
    sorted_profiles.sort();

    sorted_profiles.get((profile_index - 1) as usize).cloned()
}
