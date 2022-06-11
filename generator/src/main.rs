use color_eyre::Result;
use generator::{Api, GlProfile, GlRegistry};
use std::{fs, path::PathBuf};

const GL_XML: &str = include_str!("OpenGL-Registry/xml/gl.xml");

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut gl_registry = GlRegistry::parse(GL_XML)?;
    gl_registry.reduce(Api::Gl, 4.6, GlProfile::Core);

    let mut src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src_dir.pop();
    src_dir.push("angel/src");

    fs::write(
        src_dir.join("gl.rs"),
        gl_registry.generate(Api::Gl, 4.6, GlProfile::Core),
    )?;

    Ok(())
}
