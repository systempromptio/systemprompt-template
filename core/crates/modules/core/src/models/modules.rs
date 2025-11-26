use axum::Router;
use std::collections::HashMap;

pub use systemprompt_models::modules::{
    ApiConfig, Module, ModuleDefinition, ModulePermission, ModuleSchema, ModuleSeed, ModuleType,
    Modules, ServiceCategory,
};

#[derive(Debug)]
pub struct ModuleApiRegistry {
    registry: HashMap<String, ModuleApiImpl>,
}

#[derive(Debug)]
struct ModuleApiImpl {
    category: ServiceCategory,
    module_type: ModuleType,
    router_fn: fn(&crate::AppContext) -> Router,
    auth_required: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct ModuleApiRegistration {
    pub module_name: &'static str,
    pub category: ServiceCategory,
    pub module_type: ModuleType,
    pub router_fn: fn(&crate::AppContext) -> Router,
    pub auth_required: bool,
}

inventory::collect!(ModuleApiRegistration);

#[derive(Debug, Clone, Copy)]
pub struct WellKnownRoute {
    pub path: &'static str,
    pub handler_fn: fn(&crate::AppContext) -> Router,
    pub methods: &'static [axum::http::Method],
}

inventory::collect!(WellKnownRoute);

impl Default for ModuleApiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleApiRegistry {
    pub fn new() -> Self {
        let mut registry = HashMap::new();

        for registration in inventory::iter::<ModuleApiRegistration> {
            let api_impl = ModuleApiImpl {
                category: registration.category,
                module_type: registration.module_type,
                router_fn: registration.router_fn,
                auth_required: registration.auth_required,
            };
            registry.insert(registration.module_name.to_string(), api_impl);
        }

        Self { registry }
    }

    pub fn get_routes(&self, module_name: &str, ctx: &crate::AppContext) -> Option<Router> {
        self.registry
            .get(module_name)
            .map(|impl_| (impl_.router_fn)(ctx))
    }

    pub fn get_category(&self, module_name: &str) -> Option<ServiceCategory> {
        self.registry.get(module_name).map(|impl_| impl_.category)
    }

    pub fn get_module_type(&self, module_name: &str) -> Option<ModuleType> {
        self.registry
            .get(module_name)
            .map(|impl_| impl_.module_type)
    }

    pub fn get_auth_required(&self, module_name: &str) -> Option<bool> {
        self.registry
            .get(module_name)
            .map(|impl_| impl_.auth_required)
    }

    #[allow(private_interfaces)]
    pub fn get_registration(&self, module_name: &str) -> Option<&ModuleApiImpl> {
        self.registry.get(module_name)
    }

    pub fn modules_by_category(&self, category: ServiceCategory) -> Vec<String> {
        self.registry
            .iter()
            .filter(|(_, impl_)| matches!(impl_.category, c if c as u8 == category as u8))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

pub trait ModuleRuntime {
    fn routes(&self, ctx: &crate::AppContext, registry: &ModuleApiRegistry) -> Option<Router>;
    fn create_api_registry(&self) -> ModuleApiRegistry;
}

impl ModuleRuntime for Module {
    fn routes(&self, ctx: &crate::AppContext, registry: &ModuleApiRegistry) -> Option<Router> {
        if let Some(api) = &self.api {
            if api.enabled {
                return registry.get_routes(&self.name, ctx);
            }
        }
        None
    }

    fn create_api_registry(&self) -> ModuleApiRegistry {
        ModuleApiRegistry::new()
    }
}

impl ModuleRuntime for Modules {
    fn routes(&self, _ctx: &crate::AppContext, _registry: &ModuleApiRegistry) -> Option<Router> {
        None
    }

    fn create_api_registry(&self) -> ModuleApiRegistry {
        ModuleApiRegistry::new()
    }
}
