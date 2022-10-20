use clap::Parser;
use color_eyre::Result;
use generator::{
    generator::Generator,
    parser::{Api, GlProfile, GlRegistry},
};
use std::{fs, path::PathBuf, process::Command};

const GL_XML: &str = include_str!("OpenGL-Registry/xml/gl.xml");

/// An overly complicated opengl generator.
#[derive(Parser)]
struct Args {
    /// Path to the registry xml file, ignored if used in conjunction with --fetch.
    #[arg(short, long)]
    path: Option<PathBuf>,
    /// Fetch the latest registry xml instead of using the bundled version.
    #[arg(short, long)]
    fetch: bool,
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    // TODO: Proper error handling instead of propagating errors.
    let registry_xml: String = {
        if args.fetch {
            reqwest::blocking::get(
                "https://github.com/KhronosGroup/OpenGL-Registry/raw/main/xml/gl.xml",
            )?
            .text()?
        } else if let Some(path) = args.path {
            fs::read_to_string(path.canonicalize()?)?
        } else {
            GL_XML.to_string()
        }
    };

    let mut gl_registry = GlRegistry::parse(&registry_xml)?;
    gl_registry.reduce(Api::Gl, 4.6, GlProfile::Core);

    let mut src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src_dir.pop();
    src_dir.push("angel/src");
    let file_path = src_dir.join("gl.rs");

    println!("Writing to {}", file_path.display());

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
