//! The pure part of the pickers: matching a query against an option's words,
//! moving the keyboard highlight, and the compact context-size label a model
//! row shows.

pub fn matches(query: &str, fields: &[&str]) -> bool {
    let haystack = fields
        .iter()
        .map(|field| field.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    query
        .split_whitespace()
        .all(|word| haystack.contains(&word.to_lowercase()))
}

pub fn step(current: Option<usize>, len: usize, delta: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let Some(current) = current.filter(|index| *index < len) else {
        return Some(if delta < 0 { len - 1 } else { 0 });
    };
    let wrapped = (current as isize + delta).rem_euclid(len as isize);
    Some(wrapped as usize)
}

pub fn initial(current: Option<usize>, len: usize) -> Option<usize> {
    current
        .filter(|index| *index < len)
        .or_else(|| (len > 0).then_some(0))
}

pub fn context_size(tokens: u32, thousand: &str, million: &str) -> Option<String> {
    const THOUSAND: u32 = 1_000;
    const MILLION: u32 = 1_000_000;
    match tokens {
        0 => None,
        value if value >= MILLION => {
            let whole = value / MILLION;
            let tenth = (value % MILLION) / (MILLION / 10);
            if tenth == 0 {
                Some(format!("{whole}{million}"))
            } else {
                Some(format!("{whole}.{tenth}{million}"))
            }
        },
        value if value >= THOUSAND => {
            Some(format!("{}{thousand}", (value + THOUSAND / 2) / THOUSAND))
        },
        value => Some(value.to_string()),
    }
}
