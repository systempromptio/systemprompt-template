use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value as JsonValue;

#[derive(Debug, Copy, Clone)]
pub struct JsonParser;

impl JsonParser {
    pub fn extract_json(content: &str, custom_pattern: Option<&str>) -> Result<JsonValue> {
        if let Ok(json) = serde_json::from_str::<JsonValue>(content) {
            return Ok(json);
        }

        if let Some(pattern) = custom_pattern {
            if let Some(json) = Self::extract_with_pattern(content, pattern)? {
                return Ok(json);
            }
        }

        let patterns = vec![
            r"```json\s*\n?([\s\S]*?)\n?```",
            r"```\s*\n?([\s\S]*?)\n?```",
            r"\{[\s\S]*\}",
            r"\[[\s\S]*\]",
        ];

        for pattern in patterns {
            if let Some(json) = Self::extract_with_pattern(content, pattern)? {
                return Ok(json);
            }
        }

        if let Some(json) = Self::extract_json_heuristic(content)? {
            return Ok(json);
        }

        Err(anyhow!("No valid JSON found in response"))
    }

    fn extract_with_pattern(content: &str, pattern: &str) -> Result<Option<JsonValue>> {
        let re = Regex::new(pattern)?;

        if let Some(captures) = re.captures(content) {
            let json_str = captures
                .get(1)
                .or_else(|| captures.get(0))
                .map(|m| m.as_str())
                .ok_or_else(|| anyhow!("Pattern matched but no capture group found"))?;

            match serde_json::from_str::<JsonValue>(json_str) {
                Ok(json) => Ok(Some(json)),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn extract_json_heuristic(content: &str) -> Result<Option<JsonValue>> {
        let trimmed = content.trim();

        let Some(start_idx) = trimmed.find(['{', '[']) else {
            return Ok(None);
        };

        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut end_idx = start_idx;

        for (idx, ch) in trimmed[start_idx..].char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' | '[' if !in_string => depth += 1,
                '}' | ']' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = start_idx + idx + 1;
                        break;
                    }
                },
                _ => {},
            }
        }

        if depth == 0 && end_idx > start_idx {
            let potential_json = &trimmed[start_idx..end_idx];
            if let Ok(json) = serde_json::from_str::<JsonValue>(potential_json) {
                return Ok(Some(json));
            }
        }

        Ok(None)
    }

    pub fn clean_json_string(content: &str) -> Result<String> {
        let mut cleaned = content.trim().to_string();

        let trailing_comma_re = Regex::new(r",\s*([}\]])")?;
        cleaned = trailing_comma_re.replace_all(&cleaned, "$1").to_string();

        let single_quote_key_re = Regex::new(r"'([^']+)'\s*:")?;
        cleaned = single_quote_key_re.replace_all(&cleaned, "\"$1\":").to_string();

        let line_comment_re = Regex::new(r"//.*$")?;
        cleaned = line_comment_re.replace_all(&cleaned, "").to_string();

        let block_comment_re = Regex::new(r"/\*[\s\S]*?\*/")?;
        cleaned = block_comment_re.replace_all(&cleaned, "").to_string();

        Ok(cleaned)
    }
}
