use crate::config::{AppConfig, RenodeScriptConfig};
use crate::envsub::{envsub, EnvSubError};
use derive_more::{AsRef, Deref, Display, Into};
use std::{fs, path::Path};
use unindent::unindent;

const REPL_FILE_EXT: &str = "repl";
const RESC_PATH_PREFIX: char = '@';
const IMPORT_PATH_PREFIX: char = '<';

#[derive(Clone, Debug)]
pub struct RescDefinition {
    pub name: Name,
    pub description: Description,
    pub machine_name: MachineName,
    pub init_commands: Vec<InitCommand>,
    pub variables: Vec<Variable>,
    pub platform_descriptions: Vec<PlatformDescription>,
    pub reset: ResetMacro,
    pub start: Option<String>,
    pub pre_start_commands: Vec<PreStartCommand>,
    pub post_start_commands: Vec<PostStartCommand>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, thiserror::Error)]
pub enum RescDefinitionError {
    #[error(transparent)]
    EnvSub(#[from] EnvSubError),
    #[error(transparent)]
    RescGenericField(#[from] RescGenericFieldError),
    #[error(transparent)]
    PlatformDescription(#[from] PlatformDescriptionError),
    #[error("At least one platform description is required")]
    MissingPlatformDescription,
    #[error("The application executable file '{_0}' could not be found")]
    ExeNotFound(String),
}

impl RescDefinition {
    pub fn new<P: AsRef<Path>>(
        resc: &RenodeScriptConfig,
        _app: &AppConfig,
        bin_var_value: P,
    ) -> Result<Self, RescDefinitionError> {
        if !bin_var_value.as_ref().exists() {
            return Err(RescDefinitionError::ExeNotFound(
                bin_var_value.as_ref().display().to_string(),
            ));
        }

        let mut platform_descriptions = Vec::new();
        if let Some(p) = resc.platform_description.as_ref() {
            platform_descriptions.push(PlatformDescription::new(p.as_str())?);
        }
        for p in resc.platform_descriptions.iter() {
            platform_descriptions.push(PlatformDescription::new(p.as_str())?);
        }

        if platform_descriptions.is_empty() {
            return Err(RescDefinitionError::MissingPlatformDescription);
        }

        let mut variables = Vec::new();
        variables.push(Variable::default_binary_var(bin_var_value)?);
        for v in resc.variables.iter() {
            variables.push(Variable::new(v.as_str())?);
        }

        let mut init_commands = Vec::new();
        for c in resc.init_commands.iter() {
            init_commands.push(InitCommand::new(c.as_str())?);
        }

        let mut pre_start_commands = Vec::new();
        for c in resc.pre_start_commands.iter() {
            pre_start_commands.push(PreStartCommand::new(c.as_str())?);
        }

        let mut post_start_commands = Vec::new();
        for c in resc.post_start_commands.iter() {
            post_start_commands.push(PostStartCommand::new(c.as_str())?);
        }

        let name = resc
            .name
            .as_ref()
            .map(|s| Name::new(s))
            .unwrap_or_else(|| Ok(Name::default()))?;

        let description = resc
            .description
            .as_ref()
            .map(|s| Description::new(s))
            .unwrap_or_else(|| Ok(Description::default()))?;

        let machine_name = resc
            .machine_name
            .as_ref()
            .map(|s| MachineName::new(s))
            .unwrap_or_else(|| Ok(MachineName::default()))?;

        let reset = resc
            .reset
            .as_ref()
            .map(|s| ResetMacro::new(s))
            .unwrap_or_else(|| Ok(ResetMacro::default()))?;

        let start = resc.start.clone();

        Ok(RescDefinition {
            name,
            description,
            machine_name,
            init_commands,
            variables,
            platform_descriptions,
            reset,
            start,
            pre_start_commands,
            post_start_commands,
        })
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
pub enum PlatformDescriptionKind {
    /// A platform 'repl' file provided by Renode (begins with '@').
    #[display("renode-platform")]
    Internal,
    /// A local 'repl' file (doesn't begin with '@').
    /// Supports environment substitution in the file path.
    #[display("local-platform")]
    LocalFile,
    /// A local 'repl' file that is to be imported and generated into the output directory.
    /// The path begins with '<', the file name is provided in the data.
    /// Supports environment substitution in the file path and content.
    #[display("local-imported-platform")]
    GeneratedLocalFile(String),
    /// A string containing a platform description.
    /// Supports environment substitution in the content.
    #[display("platform")]
    String,
}

//#[display(fmt = "{}:{}", kind, content)]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PlatformDescription {
    content: String,
    kind: PlatformDescriptionKind,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, thiserror::Error)]
pub enum PlatformDescriptionError {
    #[error("The platform description is empty")]
    Empty,
    #[error("The local platform description file '{_0}' could not be found")]
    LocalFileNotFound(String),
    #[error("Encountered an IO error while reading the local file '{_0}'. {_1}")]
    Io(String, String),
    #[error("Could not determine a file name for local file '{_0}'")]
    FileName(String),
    #[error(transparent)]
    EnvSub(#[from] EnvSubError),
}

impl PlatformDescription {
    pub fn new(desc_from_config: &str) -> Result<Self, PlatformDescriptionError> {
        // TODO - indentation stuff will depend on kind
        let unindented_desc = unindent(desc_from_config.trim());
        let desc = unindented_desc.as_str();

        // Heuristic to see if this is a local repl file for desc string
        let num_lines = desc.lines().count();
        let ends_with_repl = desc.ends_with(REPL_FILE_EXT);
        let begins_with_import = desc.starts_with(IMPORT_PATH_PREFIX);

        if desc.is_empty() {
            Err(PlatformDescriptionError::Empty)
        } else if desc.starts_with(RESC_PATH_PREFIX) && ends_with_repl {
            Ok(PlatformDescription {
                content: desc.to_owned(),
                kind: PlatformDescriptionKind::Internal,
            })
        } else if num_lines == 1 && !begins_with_import && ends_with_repl {
            let local_path = envsub(desc)?;
            if Path::new(&local_path).exists() {
                Ok(PlatformDescription {
                    content: local_path,
                    kind: PlatformDescriptionKind::LocalFile,
                })
            } else {
                Err(PlatformDescriptionError::LocalFileNotFound(local_path))
            }
        } else if num_lines == 1 && begins_with_import && ends_with_repl {
            let prefix_removed = desc.trim_start_matches(IMPORT_PATH_PREFIX);
            let local_path = envsub(prefix_removed.trim())?;
            let p = Path::new(&local_path);
            if !p.exists() {
                return Err(PlatformDescriptionError::LocalFileNotFound(local_path));
            }
            let file_name = p
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| PlatformDescriptionError::FileName(local_path.clone()))?;
            let content =
                envsub(&fs::read_to_string(p).map_err(|e| {
                    PlatformDescriptionError::Io(local_path.clone(), e.to_string())
                })?)?;
            Ok(PlatformDescription {
                content,
                kind: PlatformDescriptionKind::GeneratedLocalFile(file_name.to_owned()),
            })
        } else {
            // TODO - indentation logic needs improved
            let raw_content = envsub(desc)?;
            let content = if !raw_content.starts_with("using") {
                indent::indent_by(4, &raw_content)
            } else {
                raw_content
            };
            Ok(PlatformDescription {
                content,
                kind: PlatformDescriptionKind::String,
            })
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn kind(&self) -> &PlatformDescriptionKind {
        &self.kind
    }

    pub(crate) fn resc_fmt(&self) -> String {
        match self.kind() {
            PlatformDescriptionKind::Internal => self.content().to_owned(),
            PlatformDescriptionKind::LocalFile => {
                format!("@{}", self.content())
            }
            PlatformDescriptionKind::GeneratedLocalFile(file_name) => {
                format!("@{}", file_name)
            }
            PlatformDescriptionKind::String => self.content.to_owned(),
        }
    }
}

// TODO sealed trait, AsResc or w/e that provides formated resc script syntax lines
// maybe deals with repl indentation stuff too

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, thiserror::Error)]
pub enum RescGenericFieldError {
    #[error("The field '{_0}' cannot contain an empty string")]
    Empty(&'static str),
    #[error(transparent)]
    EnvSub(#[from] EnvSubError),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct Name(String);

impl Name {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let s = envsub(v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("name"))
        } else {
            Ok(Self(s))
        }
    }
}

impl Default for Name {
    fn default() -> Self {
        Self("renode-system".to_owned())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct Description(String);

impl Description {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let s = envsub(v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("description"))
        } else {
            Ok(Self(s))
        }
    }
}

impl Default for Description {
    fn default() -> Self {
        Self("Renode script generated by renode-run".to_owned())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct MachineName(String);

impl MachineName {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let s = envsub(v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("machine-name"))
        } else {
            Ok(Self(s))
        }
    }
}

impl Default for MachineName {
    fn default() -> Self {
        Self("default-machine".to_owned())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct Variable(String);

impl Variable {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let v = unindent(v.trim());
        let s = envsub(&v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("variables"))
        } else {
            Ok(Self(s))
        }
    }

    fn default_binary_var<P: AsRef<Path>>(val: P) -> Result<Self, RescGenericFieldError> {
        let v = format!("$bin = @{}", val.as_ref().display());
        Self::new(&v)
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct ResetMacro(String);

impl ResetMacro {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let v = unindent(v.trim());
        let s = envsub(&v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("reset-macro"))
        } else {
            Ok(Self(s))
        }
    }

    pub(crate) fn resc_fmt(&self) -> String {
        indent::indent_all_by(4, &self.0)
    }
}

impl Default for ResetMacro {
    fn default() -> Self {
        Self("sysbus LoadELF $bin".to_owned())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct InitCommand(String);

impl InitCommand {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let v = unindent(v.trim());
        let s = envsub(&v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("init-commands"))
        } else {
            Ok(Self(s))
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct PreStartCommand(String);

impl PreStartCommand {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let v = unindent(v.trim());
        let s = envsub(&v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("pre-start-commands"))
        } else {
            Ok(Self(s))
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct PostStartCommand(String);

impl PostStartCommand {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let v = unindent(v.trim());
        let s = envsub(&v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Empty("post-start-commands"))
        } else {
            Ok(Self(s))
        }
    }
}
