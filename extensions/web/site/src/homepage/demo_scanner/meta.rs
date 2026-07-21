//! Shape of the static category and pillar copy the scanner joins on-disk demos
//! against.

pub(super) struct CategoryMeta {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) tagline: &'static str,
    pub(super) story: &'static str,
    pub(super) cost: &'static str,
    pub(super) feature_url: &'static str,
}

pub(super) struct PillarMeta {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) feature_url: &'static str,
    pub(super) category_ids: &'static [&'static str],
}
