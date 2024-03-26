use crate::config::AppConfig;
use crate::types::{PlatformDescriptionKind, RescDefinition};
use std::{fs, io::Write, path::Path};

pub struct RescGen<'a, T: Write> {
    writer: &'a mut T,
}

impl<'a, T: Write> RescGen<'a, T> {
    pub fn new(writer: &'a mut T) -> Self {
        RescGen { writer }
    }

    // TODO use Error type
    pub fn generate<P: AsRef<Path>>(
        mut self,
        output_dir: P,
        app: &AppConfig,
        resc: &RescDefinition,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for p in resc.platform_descriptions.iter() {
            if let PlatformDescriptionKind::GeneratedLocalFile(file_name) = p.kind() {
                let out_path = output_dir.as_ref().join(file_name);
                fs::write(out_path, p.content())?;
            }
        }

        let w = &mut self.writer;
        writeln!(w, ":name: {}", resc.name)?;
        writeln!(w, ":description: {}", resc.description)?;
        writeln!(w)?;

        if !app.omit_out_dir_path {
            writeln!(w, "path add @{}", output_dir.as_ref().display())?;
            writeln!(w)?;
        }

        if app.using_sysbus {
            writeln!(w, "using sysbus")?;
            writeln!(w)?;
        }

        writeln!(w, "mach create \"{}\"", resc.machine_name)?;
        writeln!(w)?;

        for c in resc.init_commands.iter() {
            writeln!(w, "{c}")?;
        }
        if !resc.init_commands.is_empty() {
            writeln!(w)?;
        }

        for v in resc.variables.iter() {
            writeln!(w, "{v}")?;
        }
        writeln!(w)?;

        for p in resc.platform_descriptions.iter() {
            let s = p.resc_fmt();
            match p.kind() {
                PlatformDescriptionKind::Internal
                | PlatformDescriptionKind::LocalFile
                | PlatformDescriptionKind::GeneratedLocalFile(_) => {
                    writeln!(w, "machine LoadPlatformDescription {s}")?
                }
                PlatformDescriptionKind::String => writeln!(
                    w,
                    "machine LoadPlatformDescriptionFromString\n\"\"\"\n{s}\n\"\"\""
                )?,
            }
        }
        writeln!(w)?;

        for c in resc.pre_start_commands.iter() {
            writeln!(w, "{c}")?;
        }
        if !resc.pre_start_commands.is_empty() {
            writeln!(w)?;
        }

        writeln!(w, "macro reset\n\"\"\"\n{}\n\"\"\"", resc.reset.resc_fmt())?;
        writeln!(w)?;

        writeln!(w, "runMacro $reset")?;
        writeln!(w)?;

        if !app.omit_start {
            let start_cmd = resc.start.as_deref().unwrap_or("start");
            writeln!(w, "{}", start_cmd)?;
            writeln!(w)?;

            for c in resc.post_start_commands.iter() {
                writeln!(w, "{c}")?;
            }
        }

        Ok(())
    }
}
