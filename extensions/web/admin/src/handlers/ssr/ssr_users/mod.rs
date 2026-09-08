//! The People pages: the `/admin/users` roster and the per-user detail page.
//!
//! Both are flat and URL-driven — every filter, sort, page and tab is a query
//! parameter, so any view an operator is looking at is a link they can send to
//! someone else.

mod detail;
mod roster;
mod scope_data;

// Why: the roster's own URL, which every link on both pages is built relative
// to.
pub(super) const BASE_URL: &str = "/admin/users";

pub(crate) use detail::user_detail_page as user_detail_by_id_page;
pub(crate) use roster::users_page;

use axum::extract::Query;
use axum::response::Redirect;

use crate::types::IdQuery;

// Why: the header search and a few older links still resolve a user as
// `?id=`. There is one canonical URL for a person now, so this form redirects
// to it rather than rendering a second copy of the page at a second address.
pub(crate) async fn user_detail_page(Query(params): Query<IdQuery>) -> Redirect {
    params.id().map_or_else(
        || Redirect::permanent(BASE_URL),
        |id| Redirect::permanent(&format!("{BASE_URL}/{}", urlencoding::encode(id))),
    )
}
