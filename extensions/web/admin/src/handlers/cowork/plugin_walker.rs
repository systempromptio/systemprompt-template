use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PluginFileEntry {
    pub path: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginEntry {
    pub id: String,
    pub version: String,
    pub sha256: String,
    pub files: Vec<PluginFileEntry>,
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}
