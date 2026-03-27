use super::export::PluginFile;

const RESERVED_PATH_PREFIXES: &[&str] = &["anthropic-"];
const RESERVED_PATH_SUBSTRINGS: &[&str] = &["claude-code"];
const RESERVED_BODY_WORDS: &[(&str, &str)] = &[
    ("Anthropic", "The provider"),
    ("anthropic", "the provider"),
];

pub fn sanitize_skill_md(file: &PluginFile) -> PluginFile {
    let path = strip_reserved_from_path(&file.path);
    let content = strip_frontmatter_hooks(&file.content);
    let content = rewrite_frontmatter_name(&content, &path);
    let content = replace_reserved_in_body(&content);
    PluginFile { path, content, executable: false }
}

pub fn sanitize_skill_aux(file: &PluginFile) -> PluginFile {
    PluginFile {
        path: strip_reserved_from_path(&file.path),
        content: file.content.clone(),
        executable: file.executable,
    }
}

pub fn agent_to_skill(file: &PluginFile) -> PluginFile {
    let stem = file
        .path
        .strip_prefix("agents/")
        .unwrap_or(&file.path)
        .trim_end_matches(".md");
    let path = format!("skills/{stem}/SKILL.md");
    let content = strip_frontmatter_hooks(&file.content);
    let content = replace_reserved_in_body(&content);
    PluginFile { path, content, executable: false }
}

pub fn strip_hooks_from_manifest(file: PluginFile) -> PluginFile {
    use super::export::PluginManifest;

    let content = serde_json::from_str::<PluginManifest>(&file.content)
        .map_or_else(
            |_| file.content.clone(),
            |mut m| {
                m.hooks = None;
                serde_json::to_string_pretty(&m).unwrap_or_else(|_| file.content.clone())
            },
        );

    PluginFile { path: file.path, content, executable: false }
}

fn strip_reserved_from_path(path: &str) -> String {
    let mut s = path.to_string();
    for prefix in RESERVED_PATH_PREFIXES {
        s = s.replace(&format!("skills/{prefix}"), "skills/");
    }
    for sub in RESERVED_PATH_SUBSTRINGS {
        s = s.replace(sub, "");
    }
    collapse_consecutive_hyphens(&s)
}

fn collapse_consecutive_hyphens(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        let is_hyphen = c == '-';
        if !(is_hyphen && prev) {
            out.push(c);
        }
        prev = is_hyphen;
    }
    out
}

fn find_frontmatter_bounds(lines: &[&str]) -> Option<(usize, usize)> {
    let open = lines.iter().position(|l| l.trim() == "---")?;
    let close = lines[open + 1..]
        .iter()
        .position(|l| l.trim() == "---")
        .map(|i| i + open + 1)?;
    Some((open, close))
}

fn is_top_level_yaml_key(line: &str) -> bool {
    !line.is_empty()
        && !line.starts_with(' ')
        && !line.starts_with('\t')
        && line.contains(':')
}

fn strip_frontmatter_hooks(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let Some((open, close)) = find_frontmatter_bounds(&lines) else {
        return content.to_string();
    };

    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    out.extend_from_slice(&lines[..open]);
    out.push("---");

    let mut skipping = false;
    for line in &lines[open + 1..close] {
        if is_top_level_yaml_key(line) {
            skipping = line.starts_with("hooks:");
        }
        if !skipping {
            out.push(line);
        }
    }

    out.push("---");
    out.extend_from_slice(&lines[close + 1..]);

    let mut result = out.join("\n");
    if content.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn rewrite_frontmatter_name(content: &str, path: &str) -> String {
    let skill_dir = path
        .strip_prefix("skills/")
        .and_then(|p| p.strip_suffix("/SKILL.md"))
        .unwrap_or("");
    if skill_dir.is_empty() {
        return content.to_string();
    }

    let joined = content
        .lines()
        .map(|line| {
            if line.starts_with("name:") || line.starts_with("name :") {
                format!("name: {skill_dir}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if content.ends_with('\n') {
        joined + "\n"
    } else {
        joined
    }
}

fn replace_reserved_in_body(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let Some((_, close)) = find_frontmatter_bounds(&lines) else {
        return apply_replacements(content);
    };

    let header = lines[..=close].join("\n");
    let body = lines[close + 1..].join("\n");
    let body = apply_replacements(&body);

    let mut result = header;
    result.push('\n');
    result.push_str(&body);
    if content.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn apply_replacements(text: &str) -> String {
    let mut result = text.to_string();
    for (from, to) in RESERVED_BODY_WORDS {
        result = result.replace(from, to);
    }
    result
}
