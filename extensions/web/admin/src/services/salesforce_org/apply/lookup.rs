//! Pure lookups over the query results [`permissions`](super::permissions)
//! joins.
//!
//! Apply decides what to create by matching rows it already read against the
//! spec. Those matches are ordinary functions over JSON, so they live here
//! rather than being buried inside the `async` functions that hold the
//! connection.

// JSON: Salesforce REST/Tooling query rows have no fixed schema — every lookup
// in this module matches on the raw records the SOQL projections returned.
#[must_use]
pub fn str_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
}

// Why: SOQL literals are single-quoted with backslash escapes. These values are
// Salesforce usernames and permission set API names rather than free text, but
// building a query by concatenation without escaping is the kind of thing that
// stops being true later.
#[must_use]
pub fn soql_escape(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('\'', "\\'")
}

#[must_use]
pub fn soql_list(values: &[&str]) -> String {
    values
        .iter()
        .map(|v| format!("'{}'", soql_escape(v)))
        .collect::<Vec<_>>()
        .join(",")
}

// JSON: Salesforce query rows — no fixed schema.
#[must_use]
pub fn find_app_id(apps: &[serde_json::Value], developer_name: &str) -> Option<String> {
    apps.iter()
        .find(|a| str_field(a, "DeveloperName").as_deref() == Some(developer_name))
        .and_then(|a| str_field(a, "Id"))
}

#[must_use]
pub fn find_permission_set_id(permsets: &[serde_json::Value], name: &str) -> Option<String> {
    permsets
        .iter()
        .find(|p| str_field(p, "Name").as_deref() == Some(name))
        .and_then(|p| str_field(p, "Id"))
}

#[must_use]
pub fn find_user_id(users: &[serde_json::Value], username: &str) -> Option<String> {
    users
        .iter()
        .find(|u| str_field(u, "Username").as_deref() == Some(username))
        .and_then(|u| str_field(u, "Id"))
}

// JSON: Salesforce query rows — no fixed schema.
#[must_use]
pub fn grant_exists(grants: &[serde_json::Value], permset_id: &str, app_id: &str) -> bool {
    grants.iter().any(|g| {
        str_field(g, "ParentId").as_deref() == Some(permset_id)
            && str_field(g, "SetupEntityId").as_deref() == Some(app_id)
    })
}

#[must_use]
pub fn holds_permission_set(held: &[serde_json::Value], name: &str) -> bool {
    held.iter().any(|h| {
        h.get("PermissionSet")
            .and_then(|p| str_field(p, "Name"))
            .as_deref()
            == Some(name)
    })
}
