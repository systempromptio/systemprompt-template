use std::collections::HashMap;

use super::types::{EntityEntry, SessionGroup};

pub fn enrich_session_groups(
    session_groups: &mut [SessionGroup],
    session_ratings: &[crate::types::conversation_analytics::SessionRating],
    entity_links: &[(
        String,
        crate::types::conversation_analytics::SessionEntityLink,
    )],
) {
    let ratings_map: HashMap<String, &crate::types::conversation_analytics::SessionRating> =
        session_ratings
            .iter()
            .map(|r| (r.session_id.clone(), r))
            .collect();

    let mut entities_map: HashMap<String, Vec<EntityEntry>> = HashMap::new();
    for (session_id, link) in entity_links {
        entities_map
            .entry(session_id.clone())
            .or_default()
            .push(EntityEntry {
                entity_type: link.entity_type.clone(),
                entity_name: link.entity_name.clone(),
                usage_count: link.usage_count,
            });
    }

    for group in session_groups.iter_mut() {
        enrich_single_group(group, &ratings_map, &entities_map);
    }
}

fn enrich_single_group(
    group: &mut SessionGroup,
    ratings_map: &HashMap<String, &crate::types::conversation_analytics::SessionRating>,
    entities_map: &HashMap<String, Vec<EntityEntry>>,
) {
    let entities: Vec<EntityEntry> = entities_map
        .iter()
        .filter(|(k, _)| **k == group.session_id)
        .flat_map(|(_, v)| v.clone())
        .collect();
    let entity_count = entities.len();
    let entities_preview: Vec<EntityEntry> = entities.iter().take(3).cloned().collect();
    let overflow = entity_count.saturating_sub(3);
    group.entity_count = entity_count;
    group.entities = entities;
    group.entities_preview = entities_preview;
    group.entities_overflow_count = overflow;

    let rating = ratings_map.get(&group.session_id).copied();
    if let Some(r) = rating {
        group.rating = r.rating;
        group.outcome = r.outcome.clone();
    } else {
        group.rating = 0;
        group.outcome = String::new();
    }
}
