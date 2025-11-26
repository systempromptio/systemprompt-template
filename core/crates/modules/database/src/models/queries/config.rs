use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Config module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
#[allow(clippy::enum_glob_use)]
pub const fn get_query(variant: DatabaseQueryEnum) -> Option<&'static str> {
    use DatabaseQueryEnum::*;
    match variant {
        CheckConfigTableExists => Some(include_str!(
            "../../../../config/src/queries/postgres/check_config_table_exists.sql"
        )),
        InsertModule => Some(include_str!(
            "../../../../config/src/queries/modules/postgres/insert.sql"
        )),
        GetAllModules => Some(include_str!(
            "../../../../config/src/queries/modules/postgres/list.sql"
        )),
        EnableModule => Some(include_str!(
            "../../../../config/src/queries/modules/postgres/enable.sql"
        )),
        DisableModule => Some(include_str!(
            "../../../../config/src/queries/modules/postgres/disable.sql"
        )),
        DeleteModule => Some(include_str!(
            "../../../../config/src/queries/modules/postgres/delete.sql"
        )),
        UpdateModule => Some(include_str!(
            "../../../../config/src/queries/modules/postgres/update.sql"
        )),
        CreateVariable => Some(include_str!(
            "../../../../config/src/queries/variables/postgres/insert.sql"
        )),
        GetVariable => Some(include_str!(
            "../../../../config/src/queries/variables/postgres/get.sql"
        )),
        GetVariableById => Some(include_str!(
            "../../../../config/src/queries/variables/postgres/get_by_id.sql"
        )),
        ListVariables => Some(include_str!(
            "../../../../config/src/queries/variables/postgres/list.sql"
        )),
        ListVariablesByCategory => Some(include_str!(
            "../../../../config/src/queries/variables/postgres/list_by_category.sql"
        )),
        DeleteVariable => Some(include_str!(
            "../../../../config/src/queries/variables/postgres/delete.sql"
        )),
        UpdateVariable => Some(include_str!(
            "../../../../config/src/queries/variables/postgres/update.sql"
        )),

        _ => None,
    }
}
