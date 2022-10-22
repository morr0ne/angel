use angel_generator::{
    generator::Generator,
    parser::{Api, GlProfile, GlRegistry},
};
use clap::Parser;
use color_eyre::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const GL_XML: &str = include_str!("gl.xml");

/// An overly complicated opengl generator.
#[derive(Parser)]
struct Args {
    // The folder where to put the generate files.
    #[arg(short, long)]
    out: PathBuf,
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

    let generated = Generator::generate(&gl_registry, Api::Gl, 4.6, GlProfile::Core);
    let cargo_toml = include_str!("template/Cargo.toml");
    let lib_rs = include_str!("template/lib.rs");

    let output_folder = &args.out;

    if output_folder.exists() {
        fs::remove_dir_all(output_folder)?;
    }

    fs::create_dir_all(output_folder)?;
    fs::create_dir_all(output_folder.join("src"))?;
    fs::write(output_folder.join("src/gl.rs"), generated)?;
    fs::write(output_folder.join("src/lib.rs"), lib_rs)?;
    fs::write(output_folder.join("Cargo.toml"), cargo_toml)?;
    fs::copy("LICENSE-APACHE", output_folder.join("LICENSE-APACHE"))?;
    fs::copy("LICENSE-MIT", output_folder.join("LICENSE-MIT"))?;

    // let mut src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // src_dir.pop();
    // src_dir.push("angel/src");
    // let file_path = src_dir.join("gl.rs");

    // println!("Writing to {}", file_path.display());

    // fs::write(&file_path, generated)?;

    let rustfmt_status = Command::new("cargo")
        .current_dir(output_folder)
        .arg("fmt")
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
