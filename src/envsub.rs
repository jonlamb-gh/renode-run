use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::env;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, thiserror::Error)]
pub enum EnvSubError {
    #[error("The environment variable '{0}' contains invalid unicode")]
    EnvVarNotUnicode(String),

    #[error("The environment variable '{0}' is not set and no default value is specified")]
    EnvVarNotPresent(String),
}

/// Substitute the values of environment variables.
/// Supports the following substitution style expressions:
/// * `${NAME}`
/// * `${NAME-default}`
/// * `${NAME:-default}`
pub(crate) fn envsub(input: &str) -> Result<String, EnvSubError> {
    lazy_static! {
        // Matches the following patterns with named capture groups:
        // * '${NAME}' : var = 'NAME'
        // * '${NAME-default}' : var = 'NAME', def = 'default'
        // * '${NAME:-default}' : var = 'NAME', def = 'default'
        static ref ENVSUB_RE: Regex =
            Regex::new(r"\$\{(?P<var>[a-zA-Z_][a-zA-Z0-9_]*)(:?-(?P<def>.*?))?\}")
                .expect("Could not construct envsub Regex");
    }

    replace_all(&ENVSUB_RE, input, |caps: &Captures| {
        // SAFETY: the regex requires a match for capture group 'var'
        let env_var = &caps["var"];
        match env::var(env_var) {
            Ok(env_val_val) => Ok(env_val_val),
            Err(env::VarError::NotUnicode(_)) => {
                Err(EnvSubError::EnvVarNotUnicode(env_var.to_owned()))
            }
            Err(env::VarError::NotPresent) => {
                // Use the default value if one was provided
                if let Some(def) = caps.name("def") {
                    Ok(def.as_str().to_string())
                } else {
                    Err(EnvSubError::EnvVarNotPresent(env_var.to_owned()))
                }
            }
        }
    })
}
// This is essentially a fallible version of Regex::replace_all
fn replace_all(
    re: &Regex,
    input: &str,
    replacement: impl Fn(&Captures) -> Result<String, EnvSubError>,
) -> Result<String, EnvSubError> {
    let mut new = String::with_capacity(input.len());
    let mut last_match = 0;
    for caps in re.captures_iter(input) {
        let m = caps.get(0).unwrap();
        new.push_str(&input[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&input[last_match..]);
    Ok(new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_sub_defaults() {
        assert_eq!(envsub("${NOT_SET_KEY:-foo}").unwrap(), "foo".to_owned());
        assert_eq!(envsub("${NOT_SET_KEY:-1}").unwrap(), "1".to_owned());
        assert_eq!(envsub("${NOT_SET_VAL-2}").unwrap(), "2".to_owned());
        assert_eq!(envsub("${NOT_SET_KEY-bar}").unwrap(), "bar".to_owned());
    }

    #[test]
    fn env_sub() {
        assert_eq!(
            envsub("${CARGO_PKG_VERSION}").unwrap(),
            env!("CARGO_PKG_VERSION").to_owned()
        );

        assert_eq!(
            envsub("No brackets $CARGO_PKG_VERSION").unwrap(),
            "No brackets $CARGO_PKG_VERSION".to_owned(),
        );
    }

    #[test]
    fn env_sub_errors() {
        assert_eq!(
            envsub("${NOT_SET_KEY}"),
            Err(EnvSubError::EnvVarNotPresent("NOT_SET_KEY".to_owned()))
        );
    }
}
