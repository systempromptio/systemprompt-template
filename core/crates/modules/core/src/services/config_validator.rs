use crate::services::config_manager::{DeployEnvironment, EnvironmentConfig};
use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::Path;
use systemprompt_core_logging::CliService;

#[derive(Debug, Clone, Copy)]
pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate(config: &EnvironmentConfig) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        CliService::section(&format!(
            "Validating Environment: {}",
            config.environment.as_str()
        ));

        Self::check_unresolved_variables(config, &mut report);
        Self::check_required_variables(config, &mut report);
        Self::check_empty_values(config, &mut report);
        Self::check_url_formats(config, &mut report);
        Self::check_port_values(config, &mut report);
        Self::check_environment_specific(config, &mut report);

        CliService::section("Validation Summary");

        if report.errors.is_empty() {
            CliService::success("Validation PASSED");
        } else {
            CliService::error("Validation FAILED");
            CliService::info("Errors:");
            for error in &report.errors {
                CliService::info(&format!("  - {error}"));
            }
        }

        if !report.warnings.is_empty() {
            CliService::info("Warnings:");
            for warning in &report.warnings {
                CliService::info(&format!("  - {warning}"));
            }
        }

        if !report.errors.is_empty() {
            Err(anyhow!("{} validation error(s)", report.errors.len()))
        } else {
            Ok(report)
        }
    }

    fn check_unresolved_variables(config: &EnvironmentConfig, report: &mut ValidationReport) {
        let Ok(var_regex) = Regex::new(r"\$\{[^}]+\}") else {
            report.add_error("Internal error: Invalid unresolved variable regex".to_string());
            return;
        };
        let mut unresolved = Vec::new();

        for (key, value) in &config.variables {
            if var_regex.is_match(value) {
                unresolved.push(format!("{key} = {value}"));
            }
        }

        if unresolved.is_empty() {
            CliService::success("No unresolved variables found");
        } else {
            CliService::error("Found unresolved variables:");
            for u in &unresolved {
                CliService::info(&format!("    {u}"));
                report.add_error(format!("Unresolved variable: {u}"));
            }
        }
    }

    fn check_required_variables(config: &EnvironmentConfig, report: &mut ValidationReport) {
        let required_vars = vec![
            "SERVICE_NAME",
            "SYSTEM_PATH",
            "DATABASE_URL",
            "HOST",
            "PORT",
            "API_SERVER_URL",
            "JWT_SECRET",
            "JWT_ISSUER",
        ];

        let mut missing = Vec::new();

        for var in &required_vars {
            let is_missing_or_empty = config.variables.get(*var).map_or(true, |v| v.is_empty());
            if is_missing_or_empty {
                missing.push(*var);
            }
        }

        if missing.is_empty() {
            CliService::success("All required variables present");
        } else {
            CliService::error("Required variables missing:");
            for m in &missing {
                CliService::info(&format!("    {m}"));
                report.add_error(format!("Required variable missing: {m}"));
            }
        }
    }

    fn check_empty_values(config: &EnvironmentConfig, report: &mut ValidationReport) {
        let critical_vars = vec!["DATABASE_URL", "JWT_SECRET", "ADMIN_PASSWORD"];

        let mut empty = Vec::new();

        for var in &critical_vars {
            if let Some(value) = config.variables.get(*var) {
                if value.is_empty() || value == "''" || value == "\"\"" {
                    empty.push(*var);
                }
            }
        }

        if empty.is_empty() {
            CliService::success("All critical variables have values");
        } else {
            CliService::error("Critical variables are empty:");
            for e in &empty {
                CliService::info(&format!("    {e}"));
                report.add_error(format!("Critical variable is empty: {e}"));
            }
        }
    }

    fn check_url_formats(config: &EnvironmentConfig, report: &mut ValidationReport) {
        let url_vars = vec!["DATABASE_URL", "API_SERVER_URL", "API_EXTERNAL_URL"];

        let Ok(url_regex) = Regex::new(r"^(https?|postgresql|mysql)://.*$") else {
            report.add_error("Internal error: Invalid URL regex".to_string());
            return;
        };

        let mut invalid = Vec::new();

        for url_var in &url_vars {
            if let Some(url) = config.variables.get(*url_var) {
                if !url.is_empty() && !url_regex.is_match(url) {
                    invalid.push(format!("{url_var} = {url}"));
                }
            }
        }

        if invalid.is_empty() {
            CliService::success("All URL formats are valid");
        } else {
            CliService::error("Invalid URL formats:");
            for i in &invalid {
                CliService::info(&format!("    {i}"));
                report.add_error(format!("Invalid URL format: {i}"));
            }
        }
    }

    fn check_port_values(config: &EnvironmentConfig, report: &mut ValidationReport) {
        if let Some(port_str) = config.variables.get("PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                if port == 0 {
                    CliService::error(&format!("Invalid port number: {} (must be 1-65535)", port));
                    report.add_error(format!("Invalid port number: {port}"));
                } else {
                    CliService::success(&format!("Port number is valid: {port}"));
                }
            } else {
                CliService::error(&format!("Port is not a valid number: {port_str}"));
                report.add_error(format!("Port is not a valid number: {port_str}"));
            }
        } else {
            CliService::warning("PORT not explicitly set, will use default");
            report.add_warning("PORT not explicitly set".to_string());
        }
    }

    fn check_environment_specific(config: &EnvironmentConfig, report: &mut ValidationReport) {
        match config.environment {
            DeployEnvironment::Production => {
                if let Some(use_https) = config.variables.get("USE_HTTPS") {
                    if use_https != "true" {
                        CliService::warning("Production environment should have USE_HTTPS=true");
                        report.add_warning("Production should have USE_HTTPS=true".to_string());
                    }
                }

                if let Some(rust_log) = config.variables.get("RUST_LOG") {
                    if rust_log == "debug" {
                        CliService::warning(
                            "Production environment should not have RUST_LOG=debug",
                        );
                        report.add_warning("Production should not have RUST_LOG=debug".to_string());
                    }
                }

                CliService::success(&format!(
                    "Environment-specific checks passed for: {}",
                    config.environment.as_str()
                ));
            },
            _ => {
                CliService::success(&format!(
                    "Environment-specific checks passed for: {}",
                    config.environment.as_str()
                ));
            },
        }
    }

    pub fn check_file_permissions(path: &Path, report: &mut ValidationReport) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            let perms_octal = format!("{:o}", mode & 0o777);

            if perms_octal == "644" || perms_octal == "600" {
                CliService::success(".env file has appropriate permissions");
            } else {
                CliService::warning(&format!(
                    ".env file permissions may expose secrets: {}",
                    perms_octal
                ));
                report.add_warning(format!(
                    ".env file permissions may expose secrets: {}",
                    perms_octal
                ));
            }
        }

        #[cfg(not(unix))]
        {
            CliService::warning("File permission check skipped (non-Unix system)");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}
