// Module API registration has been moved to systemprompt_core_system
// to consolidate the registration system and avoid duplication.
// Use systemprompt_core_system::register_module_api! instead.

/// Register a module
#[macro_export]
macro_rules! register_module {
    ($module_type:ty) => {
        inventory::submit! {
            Box::new($module_type) as Box<dyn $crate::Module>
        }
    };
}
