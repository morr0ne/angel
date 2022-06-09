use clap::Parser;
use color_eyre::Result;
use generator::{Api, GlProfile, GlRegistry};
use std::{fs, path::PathBuf};

/// A generator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the gl.xml file
    #[clap(short, long, conflicts_with = "fetch")]
    registry: Option<PathBuf>,
    /// Fetch the latest version from https://github.com/KhronosGroup/OpenGL-Registry/raw/main/xml/gl.xml
    #[clap(short, long, conflicts_with = "registry")]
    fetch: bool,
    /// Which api to use. One of gl, gles1, gles2 or glsc2
    #[clap(short, long, default_value = "gl")]
    api: Api,
    /// Major version of the api
    #[clap(long, default_value = "4.6")]
    version: f32,
    #[clap(long, default_value = "core")]
    profile: GlProfile,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let Args {
        registry,
        fetch,
        api,
        version,
        profile,
    } = Args::parse();

    let xml = if let Some(path) = registry {
        fs::read_to_string(path)?
    } else if fetch {
        reqwest::blocking::get(
            "https://github.com/KhronosGroup/OpenGL-Registry/raw/main/xml/gl.xml",
        )?
        .text()?
    } else {
        generator::GL_XML.to_string()
    };

    let mut gl_registry = GlRegistry::parse(&xml)?;
    gl_registry.reduce(api, version, profile);

    print!("{gl_registry}");

    Ok(())
}
