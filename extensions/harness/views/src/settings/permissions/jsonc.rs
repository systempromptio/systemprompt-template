//! Settings files are JSON with comments and trailing commas; this turns them
//! into plain JSON.

pub fn strip_jsonc(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    strip_trailing_commas(&strip_comments(&chars))
}

fn strip_comments(chars: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(chars.len());
    let mut index = 0;
    let mut in_string = false;
    while let Some(&current) = chars.get(index) {
        let next = chars.get(index + 1).copied();
        if in_string {
            out.push(current);
            if current == '\\' {
                out.extend(next);
                index += 1;
            } else if current == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        match (current, next) {
            ('/', Some('/')) => {
                while chars.get(index).is_some_and(|character| *character != '\n') {
                    index += 1;
                }
            },
            ('/', Some('*')) => {
                index += 2;
                while index < chars.len()
                    && !(chars.get(index) == Some(&'*') && chars.get(index + 1) == Some(&'/'))
                {
                    index += 1;
                }
                index += 2;
            },
            _ => {
                in_string = current == '"';
                out.push(current);
                index += 1;
            },
        }
    }
    out
}

fn strip_trailing_commas(chars: &[char]) -> String {
    let mut out = String::with_capacity(chars.len());
    let mut in_string = false;
    let mut escaped = false;
    for (index, &current) in chars.iter().enumerate() {
        if in_string {
            in_string = escaped || current != '"';
            escaped = !escaped && current == '\\';
            out.push(current);
            continue;
        }
        if current == ',' {
            let closes = chars[index + 1..]
                .iter()
                .find(|character| !character.is_whitespace());
            if matches!(closes, Some('}' | ']')) {
                continue;
            }
        }
        in_string = current == '"';
        out.push(current);
    }
    out
}
