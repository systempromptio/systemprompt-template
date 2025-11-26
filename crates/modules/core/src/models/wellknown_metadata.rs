#[derive(Debug, Clone, Copy)]
pub struct WellKnownMetadata {
    pub path: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

inventory::collect!(WellKnownMetadata);

impl WellKnownMetadata {
    pub const fn new(path: &'static str, name: &'static str, description: &'static str) -> Self {
        Self {
            path,
            name,
            description,
        }
    }
}

pub fn get_wellknown_metadata(path: &str) -> Option<WellKnownMetadata> {
    inventory::iter::<WellKnownMetadata>
        .into_iter()
        .find(|meta| meta.path == path)
        .copied()
}
