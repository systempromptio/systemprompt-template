//! The static registry of console nav items: every page the console can link
//! from its sidebar or home cards, the route it lives at, and the console
//! feature that compiles it. A dashboard config names items by id; the
//! registry is what turns an id into a link, and what refuses an id whose
//! page this build left out.

mod account;
mod activity;
mod people;
mod platform;

#[derive(Debug, Clone, Copy)]
pub(crate) struct NavItemDef {
    pub(crate) id: &'static str,
    pub(crate) href: &'static str,
    pub(crate) text: &'static str,
    pub(crate) leaf: &'static str,
    pub(crate) child: &'static str,
    pub(crate) child2: &'static str,
    pub(crate) scope_nav: bool,
    pub(crate) console: bool,
    pub(crate) external: bool,
    pub(crate) attention: bool,
    pub(crate) needs: Option<&'static str>,
    pub(crate) compiled: bool,
    pub(crate) icon: &'static str,
}

pub(crate) fn all() -> impl Iterator<Item = &'static NavItemDef> {
    [
        activity::ITEMS,
        people::ITEMS,
        platform::ITEMS,
        account::ITEMS,
    ]
    .into_iter()
    .flatten()
}

pub(crate) fn find(id: &str) -> Option<&'static NavItemDef> {
    all().find(|item| item.id == id)
}
