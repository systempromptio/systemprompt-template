use std::collections::HashMap;

use crate::types::access_control::AccessControlRule;
use crate::types::DepartmentStats;
use serde_json::json;

pub(crate) struct EntityJsonParams<'a> {
    pub(crate) id: &'a str,
    pub(crate) name: &'a str,
    pub(crate) description: &'a str,
    pub(crate) enabled: bool,
    pub(crate) entity_type: &'a str,
    pub(crate) yaml_roles: Option<&'a [String]>,
}

pub(crate) fn build_rules_map(
    all_rules: &[AccessControlRule],
) -> HashMap<(String, String), Vec<&AccessControlRule>> {
    let mut rules_map: HashMap<(String, String), Vec<&AccessControlRule>> = HashMap::new();
    for rule in all_rules {
        rules_map
            .entry((rule.entity_type.clone(), rule.entity_id.clone()))
            .or_default()
            .push(rule);
    }
    rules_map
}

pub(crate) fn build_departments_json(departments: &[DepartmentStats]) -> Vec<serde_json::Value> {
    departments
        .iter()
        .map(|d| {
            json!({
                "department": d.department,
                "user_count": d.user_count,
                "active_count": d.active_count,
            })
        })
        .collect()
}

pub(crate) fn build_entity_json(
    params: &EntityJsonParams<'_>,
    rules_map: &HashMap<(String, String), Vec<&AccessControlRule>>,
    known_roles: &[&str],
    dept_names: &[&str],
    departments: &[DepartmentStats],
) -> serde_json::Value {
    let key = (params.entity_type.to_string(), params.id.to_string());
    let entity_rules = rules_map.get(&key);

    let (roles, role_count) = build_roles_json(entity_rules, known_roles, params.yaml_roles);
    let (dept_assignments, dept_count) = build_dept_json(entity_rules, dept_names, departments);

    json!({
        "id": params.id,
        "name": params.name,
        "description": params.description,
        "enabled": params.enabled,
        "entity_type": params.entity_type,
        "roles": roles,
        "departments": dept_assignments,
        "role_count": role_count,
        "department_count": dept_count,
        "total_departments": dept_names.len(),
    })
}

fn build_roles_json(
    entity_rules: Option<&Vec<&AccessControlRule>>,
    known_roles: &[&str],
    yaml_roles: Option<&[String]>,
) -> (Vec<serde_json::Value>, usize) {
    let mut role_count = 0;
    let roles: Vec<serde_json::Value> = known_roles
        .iter()
        .map(|role_name| {
            let from_yaml = yaml_roles.is_some_and(|yr| yr.iter().any(|r| r == role_name));
            let from_db = entity_rules.is_some_and(|rules| {
                rules.iter().any(|r| {
                    r.rule_type == "role" && r.rule_value == *role_name && r.access == "allow"
                })
            });
            let assigned = from_yaml || from_db;
            if assigned {
                role_count += 1;
            }
            json!({
                "name": role_name,
                "assigned": assigned,
            })
        })
        .collect();
    (roles, role_count)
}

fn build_dept_json(
    entity_rules: Option<&Vec<&AccessControlRule>>,
    dept_names: &[&str],
    departments: &[DepartmentStats],
) -> (Vec<serde_json::Value>, usize) {
    let mut dept_count = 0;
    let dept_assignments: Vec<serde_json::Value> = dept_names
        .iter()
        .map(|dept_name| {
            let rule = entity_rules.and_then(|rules| {
                rules
                    .iter()
                    .find(|r| r.rule_type == "department" && r.rule_value == *dept_name)
            });
            let assigned = rule.is_some_and(|r| r.access == "allow");
            let default_included = rule.is_some_and(|r| r.default_included);
            let user_count = departments
                .iter()
                .find(|d| d.department == *dept_name)
                .map_or(0, |d| d.user_count);
            if assigned {
                dept_count += 1;
            }
            json!({
                "name": dept_name,
                "assigned": assigned,
                "default_included": default_included,
                "user_count": user_count,
            })
        })
        .collect();
    (dept_assignments, dept_count)
}
