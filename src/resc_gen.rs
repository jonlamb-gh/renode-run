use crate::config::AppConfig;
use crate::types::{PlatformDescriptionKind, RescDefinition};
use std::io::Write;

pub struct RescGen<'a, T: Write> {
    writer: &'a mut T,
}

impl<'a, T: Write> RescGen<'a, T> {
    pub fn new(writer: &'a mut T) -> Self {
        RescGen { writer }
    }

    // TODO use Error type
    pub fn generate(
        mut self,
        app: &AppConfig,
        resc: &RescDefinition,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let w = &mut self.writer;
        writeln!(w, ":name: {}", resc.name)?;
        writeln!(w, ":description: {}", resc.description)?;
        writeln!(w)?;

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
            writeln!(w, "start")?;
            writeln!(w)?;

            for c in resc.post_start_commands.iter() {
                writeln!(w, "{c}")?;
            }
        }

        Ok(())
    }
}
