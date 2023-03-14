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
    pub platform_descriptions: Vec<String>,
    pub reset: Option<String>,
    pub pre_start_commands: Vec<String>,
    pub post_start_commands: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct RenodeCliConfig {
    pub disable_xwt: bool,
    // TODO add them all
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct AppConfig {
    // TODO not used atm
    pub use_relative_paths: bool,
    pub resc_file_name: Option<String>,
    pub disable_envsub: bool,
    pub using_sysbus: bool,
    pub omit_start: bool,
    pub environment_variables: Vec<(String, String)>,
    pub renode: Option<String>,
}
