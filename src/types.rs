use crate::config::{AppConfig, RenodeScriptConfig};
use crate::envsub::{envsub, EnvSubError};
use derive_more::{AsRef, Deref, Display, Into};
use std::path::Path;
use unindent::unindent;

const RESC_PATH_PREFIX: char = '@';
const REPL_FILE_EXT: &str = "repl";

#[derive(Clone, Debug)]
pub struct RescDefinition {
    pub name: Name,
    pub description: Description,
    pub machine_name: MachineName,
    pub init_commands: Vec<InitCommand>,
    pub variables: Vec<Variable>,
    pub platform_descriptions: Vec<PlatformDescription>,
    pub reset: ResetMacro,
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

        if resc.platform_descriptions.is_empty() {
            return Err(RescDefinitionError::MissingPlatformDescription);
        }

        let mut platform_descriptions = Vec::new();
        for p in resc.platform_descriptions.iter() {
            platform_descriptions.push(PlatformDescription::new(p.as_str())?);
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

        Ok(RescDefinition {
            name,
            description,
            machine_name,
            init_commands,
            variables,
            platform_descriptions,
            reset,
            pre_start_commands,
            post_start_commands,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display)]
pub enum PlatformDescriptionKind {
    /// A platform 'repl' file provided by Renode (begins with '@')
    #[display(fmt = "renode-platform")]
    Internal,
    /// A local 'repl' file
    #[display(fmt = "local-platform")]
    LocalFile,
    /// A string containing a platform description
    #[display(fmt = "platform")]
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
    Emtpy,
    #[error("The local platform description file '{_0}' could not be found")]
    LocalFileNotFound(String),
    #[error(transparent)]
    EnvSub(#[from] EnvSubError),
}

impl PlatformDescription {
    pub fn new(desc: &str) -> Result<Self, PlatformDescriptionError> {
        // TODO
        let desc = unindent(desc.trim());
        let desc = &desc;

        // Heuristic to see if this is a local repl file for desc string
        let num_lines = desc.lines().count();
        let ends_with_repl = desc.ends_with(REPL_FILE_EXT);

        if desc.is_empty() {
            Err(PlatformDescriptionError::Emtpy)
        } else if desc.starts_with(RESC_PATH_PREFIX) && ends_with_repl {
            Ok(PlatformDescription {
                content: desc.to_owned(),
                kind: PlatformDescriptionKind::Internal,
            })
        } else if num_lines == 1 && ends_with_repl {
            let local_path = envsub(desc)?;
            // TODO
            // - deal with relative vs absolute paths, make everything absolute
            // - deal with '@' RESC_PATH_PREFIX on relative and absolute paths
            if Path::new(&local_path).exists() {
                Ok(PlatformDescription {
                    content: local_path,
                    kind: PlatformDescriptionKind::LocalFile,
                })
            } else {
                Err(PlatformDescriptionError::LocalFileNotFound(local_path))
            }
        } else {
            let content = envsub(desc)?;
            Ok(PlatformDescription {
                content,
                kind: PlatformDescriptionKind::String,
            })
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn kind(&self) -> PlatformDescriptionKind {
        self.kind
    }

    pub(crate) fn resc_fmt(&self) -> String {
        match self.kind() {
            PlatformDescriptionKind::Internal | PlatformDescriptionKind::LocalFile => {
                self.content().to_owned()
            }
            PlatformDescriptionKind::String => indent::indent_by(4, self.content()),
        }
    }
}

// TODO sealed trait, AsResc or w/e that provides formated resc script syntax lines
// maybe deals with repl indentation stuff too

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, thiserror::Error)]
pub enum RescGenericFieldError {
    #[error("The field '{_0}' cannot contain an empty string")]
    Emtpy(&'static str),
    #[error(transparent)]
    EnvSub(#[from] EnvSubError),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, AsRef, Deref, Display, Into)]
pub struct Name(String);

impl Name {
    pub fn new(v: &str) -> Result<Self, RescGenericFieldError> {
        let s = envsub(v)?;
        if s.is_empty() {
            Err(RescGenericFieldError::Emtpy("name"))
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
            Err(RescGenericFieldError::Emtpy("description"))
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
            Err(RescGenericFieldError::Emtpy("machine-name"))
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
            Err(RescGenericFieldError::Emtpy("variables"))
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
            Err(RescGenericFieldError::Emtpy("reset-macro"))
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
            Err(RescGenericFieldError::Emtpy("init-commands"))
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
            Err(RescGenericFieldError::Emtpy("pre-start-commands"))
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
            Err(RescGenericFieldError::Emtpy("post-start-commands"))
        } else {
            Ok(Self(s))
        }
    }
}
