use itertools::Itertools;

use crate::parser::{Api, GlProfile, GlRegistry};

pub struct Generator;

impl Generator {
    pub const fn types() -> &'static str {
        r#"#[cfg(not(feature = "std"))]
        use core::ffi::{c_char, c_double, c_float, c_int, c_short, c_uchar, c_uint, c_ushort, c_void};
        
        #[cfg(feature = "std")]
        use std::os::raw::{
            c_char, c_double, c_float, c_int, c_short, c_uchar, c_uint, c_ushort, c_void,
        };
        
        pub type GLvoid = c_void;
        pub type GLbyte = c_char;
        pub type GLubyte = c_uchar;
        pub type GLchar = c_char;
        pub type GLboolean = c_uchar;
        pub type GLshort = c_short;
        pub type GLushort = c_ushort;
        pub type GLint = c_int;
        pub type GLuint = c_uint;
        pub type GLint64 = i64;
        pub type GLuint64 = u64;
        pub type GLintptr = isize;
        pub type GLsizeiptr = isize;
        pub type GLintptrARB = isize;
        pub type GLsizeiptrARB = isize;
        pub type GLint64EXT = i64;
        pub type GLuint64EXT = u64;
        pub type GLsizei = GLint;
        pub type GLclampx = c_int;
        pub type GLfixed = GLint;
        pub type GLhalf = c_ushort;
        pub type GLhalfNV = c_ushort;
        pub type GLhalfARB = c_ushort;
        pub type GLenum = c_uint;
        pub type GLbitfield = c_uint;
        pub type GLfloat = c_float;
        pub type GLdouble = c_double;
        pub type GLclampf = c_float;
        pub type GLclampd = c_double;
        pub type GLcharARB = c_char;
        #[cfg(target_os = "macos")]
        pub type GLhandleARB = *const c_void;
        #[cfg(not(target_os = "macos"))]
        pub type GLhandleARB = c_uint;
        pub enum __GLsync {}
        pub type GLsync = *const __GLsync;
        pub enum _cl_context {}
        pub enum _cl_event {}
        pub type GLvdpauSurfaceNV = GLintptr;
        pub type GLeglClientBufferEXT = *const c_void;
        pub type GLeglImageOES = *const c_void;
        pub type GLDEBUGPROC = extern "system" fn(
            source: GLenum,
            type_: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut c_void,
        );
        pub type GLDEBUGPROCARB = extern "system" fn(
            source: GLenum,
            type_: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut c_void,
        );
        pub type GLDEBUGPROCKHR = extern "system" fn(
            source: GLenum,
            type_: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut GLvoid,
        );
        pub type GLDEBUGPROCAMD = extern "system" fn(
            id: GLuint,
            category: GLenum,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut GLvoid,
        );
        pub type GLVULKANPROCNV = extern "system" fn();"#
    }

    pub fn generate(registry: &GlRegistry, api: Api, version: f32, profile: GlProfile) -> String {
        let formated_enums = &registry.gl_enums.iter().format_with("\n", |gl_enum, f| {
            let enum_type = if gl_enum.bitmask {
                "GLbitfield"
            } else if gl_enum.value == "0xFFFFFFFFFFFFFFFF" {
                "u64"
            } else {
                "GLenum"
            };

            f(&format_args!(
                "pub const {enum_name}: {enum_type} = {enum_value};",
                enum_name = gl_enum.name,
                enum_value = gl_enum.value
            ))
        });

        let formated_fields = &registry.gl_commands.iter().format_with(",\n", |gl_command, f| {
            f(&format_args!(
                r#"{command_name}: extern "system" fn({function_parameters}){function_return_type}"#,
                command_name=gl_command.name,
                function_parameters=&gl_command
                    .gl_params
                    .iter()
                    .format_with(",", |gl_param, f| f(&gl_param.gl_type)),
                function_return_type=gl_command.return_type
            ))
        });

        let formated_constructor =
            &registry
                .gl_commands
                .iter()
                .format_with(",\n", |gl_command, f| {
                    f(&format_args!(
                        // NOTE: Thise needs to be on multiple lines otherwise the formater fricks out. I'd probably be a good idea to report the bug.
                        r#"{command_name}: transmute::<*const c_void, 
                extern "system" fn({function_parameters}){function_return_type}>
                (load_pointer(b"{command_name}\0")?)"#,
                        command_name = gl_command.name,
                        function_parameters = &gl_command
                            .gl_params
                            .iter()
                            .format_with(",", |gl_param, f| f(&gl_param.gl_type)),
                        function_return_type = gl_command.return_type,
                    ))
                });

        let formated_methods = &registry.gl_commands.iter().format_with("\n", |gl_command, f| {
            let mut brackets = String::new();

            for gl_param in &gl_command.gl_params {
                match gl_param.gl_type.as_str() {
                    "GLenum" => brackets.push_str("{:#X}, "),
                    gl_type => {
                        if gl_type.contains('*') {
                            brackets.push_str("{:p}, ")
                        } else {
                            brackets.push_str("{:?}, ")
                        }
                    }
                }
            }

            brackets.pop();
            brackets.pop();

            f(&format_args!(
                r#"pub unsafe fn {function_name}(&self,{function_parameters}){function_return_type}
                {{
                    #[cfg(all(debug_assertions, feature = "tracing", feature = "trace-calls"))]
                    trace!("Calling gl{function_name}({brackets})", {trace_parameters});
                    (self.{inner_function})({inner_function_parameters})
                }}"#,
                function_name = gl_command.name.replacen("gl", "", 1),
                function_parameters =
                    &gl_command
                        .gl_params
                        .iter()
                        .format_with(",", |gl_param, f| f(&format_args!(
                            "{}:{}",
                            gl_param.name, gl_param.gl_type
                        ))),
                function_return_type = gl_command.return_type,
                inner_function = gl_command.name,
                trace_parameters = &gl_command.gl_params.iter().format_with(",", |gl_param, f| {
                    if gl_param.gl_type == "GLDEBUGPROC" {
                        f(&format_args!(
                            "transmute::<_, Option<fn()>>({})",
                            gl_param.name
                        ))
                    } else {
                        f(&gl_param.name)
                    }
                }),
                inner_function_parameters = &gl_command
                    .gl_params
                    .iter()
                    .format_with(",", |gl_param, f| f(&gl_param.name)),
            ))
        });

        format!(
            r#"
#![allow(bad_style)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::upper_case_acronyms)]

#[cfg(not(feature = "std"))]
use core::{{
    ffi::{{c_void, CStr}},
    fmt::Display,
    mem::transmute,
}};
#[cfg(feature = "std")]
use std::{{ffi::CStr, fmt::Display, mem::transmute, os::raw::c_void}};


#[cfg(all(feature = "tracing", feature = "trace-calls"))]
use tracing::{{error, trace}};

pub type Result<T, E = LoadError> = core::result::Result<T, E>;

#[derive(Debug)]
pub struct LoadError {{
    pub name: &'static str,
    pub pointer: usize,
}}

#[cfg(feature = "std")]
impl std::error::Error for LoadError {{}}

impl Display for LoadError {{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
        write!(
            f,
            "Failed to load function \"{{}}\", expected a valid pointer instead got {{}}",
            self.name, self.pointer
        )
    }}
}}

pub use types::*;
pub mod types {{
    {types}
}}

pub use enums::*;
pub mod enums {{
use super::*;
{formated_enums}
}}

pub struct Gl {{
{formated_fields}
}}

impl Gl {{
    pub unsafe fn load<F>(mut loader_function: F) -> Result<Self>
    where
        F: FnMut(&CStr) -> *const c_void,
    {{
        let mut load_pointer = |name: &'static [u8]| -> Result<*const c_void> {{
            let pointer = loader_function(CStr::from_bytes_with_nul_unchecked(name));
            let pointer_usize = pointer as usize;

            if pointer_usize == core::usize::MAX || pointer_usize < 8 {{
                Err(LoadError {{
                    name: core::str::from_utf8_unchecked(&name[..name.len() - 1]),
                    pointer: pointer_usize,
                }})
            }} else {{
                Ok(pointer)
            }}
        }};

        Ok(Self {{
            {formated_constructor}
        }})
    }}

    {formated_methods}
}}"#,
            types = Self::types()
        )
    }
}
