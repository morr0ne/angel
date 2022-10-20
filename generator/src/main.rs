use color_eyre::Result;
use generator::{
    generator::Generator,
    parser::{Api, GlProfile, GlRegistry},
};
use std::{fs, path::PathBuf, process::Command};

const GL_XML: &str = include_str!("OpenGL-Registry/xml/gl.xml");

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut gl_registry = GlRegistry::parse(GL_XML)?;
    gl_registry.reduce(Api::Gl, 4.6, GlProfile::Core);

    let mut src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src_dir.pop();
    src_dir.push("angel/src");
    let file_path = src_dir.join("gl.rs");

    fs::write(
        &file_path,
        Generator::generate(&gl_registry, Api::Gl, 4.6, GlProfile::Core),
    )?;

    let rustfmt_status = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(&file_path)
        .status();

    if let Ok(status) = rustfmt_status {
        if status.success() {
            println!("Runned rustfmt on generated code successfully")
        } else {
            println!("Failed to format code")
        }
    } else {
        println!("Failed to format code")
    }

    Ok(())
}
