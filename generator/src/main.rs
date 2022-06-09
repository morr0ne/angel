use color_eyre::Result;
use generator::GlRegistry;

const GL_XML: &str = include_str!("OpenGL-Registry/xml/gl.xml");

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut gl_registry = GlRegistry::parse(GL_XML)?;
    // gl_registry.reduce(api, version, profile);

    print!("{gl_registry}");

    Ok(())
}
