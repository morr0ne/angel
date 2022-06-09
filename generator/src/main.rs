use color_eyre::Result;
use generator::{Api, GlProfile, GlRegistry};

const GL_XML: &str = include_str!("OpenGL-Registry/xml/gl.xml");

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut gl_registry = GlRegistry::parse(GL_XML)?;
    gl_registry.reduce(Api::Gl, 4.6, GlProfile::Core);

    print!("{gl_registry}");

    Ok(())
}
