use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct RenodeRunConfig {
    #[serde(flatten)]
    pub resc: RenodeScriptConfig,

    #[serde(flatten)]
    pub cli: RenodeCliConfig,

    #[serde(flatten)]
    pub app: AppConfig,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct RenodeScriptConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub machine_name: Option<String>,
    pub init_commands: Vec<String>,
    pub variables: Vec<String>,
    pub platform_description: Option<String>,
    pub platform_descriptions: Vec<String>,
    pub reset: Option<String>,
    pub pre_start_commands: Vec<String>,
    pub post_start_commands: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct RenodeCliConfig {
    pub plain: bool,
    pub port: Option<u16>,
    pub disable_xwt: bool,
    pub hide_monitor: bool,
    pub hide_log: bool,
    pub hide_analyzers: bool,
    pub console: bool,
    pub keep_temporary_files: bool,
}

impl RenodeCliConfig {
    pub(crate) fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        if self.plain {
            args.push("--plain".to_owned());
        }
        if let Some(p) = self.port {
            args.push("--port".to_owned());
            args.push(p.to_string());
        }
        if self.disable_xwt {
            args.push("--disable-xwt".to_owned());
        }
        if self.hide_monitor {
            args.push("--hide-monitor".to_owned());
        }
        if self.hide_log {
            args.push("--hide-log".to_owned());
        }
        if self.hide_analyzers {
            args.push("--hide-analyzers".to_owned());
        }
        if self.console {
            args.push("--console".to_owned());
        }
        if self.keep_temporary_files {
            args.push("--keep-temporary-files".to_owned());
        }
        args
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct AppConfig {
    // TODO use these
    pub use_relative_paths: bool,
    pub resc_file_name: Option<String>,
    pub disable_envsub: bool,
    pub using_sysbus: bool,
    pub omit_start: bool,
    pub environment_variables: Vec<(String, String)>,
    pub renode: Option<String>,
    pub omit_out_dir_path: bool,
}
