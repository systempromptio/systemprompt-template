pub fn generate_slug(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '_' || c == '.' || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>();

    let cleaned = collapse_consecutive_hyphens(&slug);
    cleaned.trim_matches('-').to_string()
}

fn collapse_consecutive_hyphens(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_hyphen = false;

    for c in s.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    result
}

pub fn generate_unique_slug(name: &str, existing_slugs: &[String]) -> String {
    let base_slug = generate_slug(name);

    if !existing_slugs.contains(&base_slug) {
        return base_slug;
    }

    for i in 1..1000 {
        let candidate = format!("{base_slug}-{i}");
        if !existing_slugs.contains(&candidate) {
            return candidate;
        }
    }

    let uuid_str = uuid::Uuid::new_v4().simple().to_string();
    format!("{}-{}", base_slug, &uuid_str[..8])
}
