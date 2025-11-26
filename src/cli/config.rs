use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    Quiet,
    Normal,
    Verbose,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub output_format: OutputFormat,
    pub verbosity: VerbosityLevel,
    pub color_mode: ColorMode,
    pub interactive: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Table,
            verbosity: VerbosityLevel::Normal,
            color_mode: ColorMode::Auto,
            interactive: true,
        }
    }
}

impl CliConfig {
    pub fn new() -> Self {
        let mut config = Self::default();
        config.apply_environment_variables();
        config
    }

    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn with_verbosity(mut self, level: VerbosityLevel) -> Self {
        self.verbosity = level;
        self
    }

    pub fn with_color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    fn apply_environment_variables(&mut self) {
        if let Ok(format) = env::var("SYSTEMPROMPT_OUTPUT_FORMAT") {
            self.output_format = match format.to_lowercase().as_str() {
                "json" => OutputFormat::Json,
                "yaml" => OutputFormat::Yaml,
                "table" => OutputFormat::Table,
                _ => self.output_format,
            };
        }

        if let Ok(level) = env::var("SYSTEMPROMPT_LOG_LEVEL") {
            self.verbosity = match level.to_lowercase().as_str() {
                "quiet" => VerbosityLevel::Quiet,
                "normal" => VerbosityLevel::Normal,
                "verbose" => VerbosityLevel::Verbose,
                "debug" => VerbosityLevel::Debug,
                _ => self.verbosity,
            };
        }

        if env::var("SYSTEMPROMPT_NO_COLOR").is_ok() || env::var("NO_COLOR").is_ok() {
            self.color_mode = ColorMode::Never;
        }

        if env::var("SYSTEMPROMPT_NON_INTERACTIVE").is_ok() {
            self.interactive = false;
        }
    }

    pub fn should_use_color(&self) -> bool {
        match self.color_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        }
    }

    pub fn is_json_output(&self) -> bool {
        self.output_format == OutputFormat::Json
    }

    pub fn should_show_verbose(&self) -> bool {
        self.verbosity >= VerbosityLevel::Verbose
    }
}

thread_local! {
    static CLI_CONFIG: std::cell::RefCell<CliConfig> = std::cell::RefCell::new(CliConfig::new());
}

pub fn set_global_config(config: CliConfig) {
    CLI_CONFIG.with(|c| {
        *c.borrow_mut() = config;
    });
}

pub fn get_global_config() -> CliConfig {
    CLI_CONFIG.with(|c| c.borrow().clone())
}
