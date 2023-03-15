use crate::config::RenodeRunConfig;
use crate::opts::Opts;
use crate::resc_gen::RescGen;
use crate::types::{PlatformDescriptionKind, RescDefinition};
use clap::Parser;
use serde::Deserialize;
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

mod config;
mod envsub;
mod opts;
mod resc_gen;
mod types;

#[derive(Clone, Debug, Deserialize, Default)]
struct CargoPackageMetadata {
    pub renode: Option<RenodeRunConfig>,
}

// TODO error types and printing stuff
fn main() {
    let opts = Opts::parse();

    env_logger::init();

    let input_file = if let Some(manual_input_file) = opts.config.as_ref() {
        log::debug!("Using config '{}'", manual_input_file.display());
        manual_input_file.clone()
    } else {
        log::debug!("Looking up default config from cargo metadata");
        let cmd = cargo_metadata::MetadataCommand::new();
        let _metadata = cmd.exec().unwrap();

        // TODO deal with workspaces/selected-package-from-available-ones ...

        PathBuf::from("Cargo.toml")
    };

    let manifest_bytes = fs::read(input_file).unwrap();

    let manifest =
        cargo_toml::Manifest::<CargoPackageMetadata>::from_slice_with_metadata(&manifest_bytes)
            .unwrap();

    let renode_config = manifest
        .package
        .and_then(|p| p.metadata)
        .and_then(|md| md.renode)
        .unwrap_or_default();

    for (env_var, env_val) in renode_config.app.environment_variables.iter() {
        env::set_var(env_var, env_val);
    }

    let tmpdir = tempfile::tempdir().unwrap();
    let output_dir = opts
        .output_dir
        .unwrap_or_else(|| tmpdir.path().join("renode-run"));

    log::debug!("Using output dir '{}'", output_dir.display());
    fs::create_dir_all(&output_dir).unwrap();

    let resc_def =
        RescDefinition::new(&renode_config.resc, &renode_config.app, &opts.input).unwrap();

    let output_file_path = renode_config
        .app
        .resc_file_name
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| output_dir.join("emulate.resc"));

    for p in resc_def.platform_descriptions.iter() {
        if let PlatformDescriptionKind::GeneratedLocalFile(file_name) = p.kind() {
            let out_path = output_dir.join(file_name);
            fs::write(out_path, p.content()).unwrap();
        }
    }

    log::debug!("Using output script '{}'", output_file_path.display());

    let mut output_file = std::fs::File::create(&output_file_path).unwrap();
    let resc_gen = RescGen::new(&mut output_file);
    resc_gen.generate(&renode_config.app, &resc_def).unwrap();
    output_file.sync_all().unwrap();
    drop(output_file);

    if !opts.no_run {
        let default_bin = PathBuf::from("renode");
        let cfg_bin = renode_config
            .app
            .renode
            .as_ref()
            .map(|s| envsub::envsub(s).unwrap());

        let renode_bin = if let Some(opts_bin) = opts.renode_bin.as_ref() {
            opts_bin.clone()
        } else if let Some(cfgb) = cfg_bin {
            PathBuf::from(cfgb)
        } else {
            default_bin
        };

        log::debug!("Using renode bin '{}'", renode_bin.display());
        let mut args = renode_config.cli.to_args();
        args.insert(0, output_file_path.display().to_string());
        let mut child = Command::new(renode_bin)
            .args(args)
            .envs(renode_config.app.environment_variables)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn()
            .expect("Failed to start renode process");
        let _exit_status = child.wait().unwrap();
    }
}
