use roxmltree::Document;
use std::{collections::HashSet, str::FromStr};

/// A list of all keywords reserved by the language.
const KEYWORDS: [&str; 51] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Api {
    Gl,
    Gles1,
    Gles2,
    Glsc2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlProfile {
    Core,
    Compatibility,
    Common,
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid api")]
pub struct GlProfileFromStrError;

impl FromStr for GlProfile {
    type Err = GlProfileFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "core" => Ok(Self::Core),
            "compatibility" => Ok(Self::Compatibility),
            "common" => Ok(Self::Common),
            _ => Err(GlProfileFromStrError),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid api")]
pub struct ApiFromStrError;

impl FromStr for Api {
    type Err = ApiFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gl" => Ok(Self::Gl),
            "gles1" => Ok(Self::Gles1),
            "gles2" => Ok(Self::Gles2),
            "glsc2" => Ok(Self::Glsc2),
            _ => Err(ApiFromStrError),
        }
    }
}

impl Api {
    pub const fn api(&self) -> &str {
        match self {
            Api::Gl => "gl",
            Api::Gles1 => "gles1",
            Api::Gles2 => "gles2",
            Api::Glsc2 => "glsc2",
        }
    }
}

pub struct GlRegistry {
    pub gl_enums: Vec<GlEnum>,
    pub gl_commands: Vec<GlCommand>,
    pub gl_features: Vec<GlFeature>,
    pub gl_extensions: Vec<GlExtension>,
}

pub struct GlEnum {
    pub name: String,
    pub value: String,
    pub bitmask: bool,
    pub group: Option<String>,
}

pub struct GlCommand {
    pub name: String,
    pub gl_params: Vec<GlParam>,
    pub return_type: String,
}

pub struct GlParam {
    pub gl_type: String,
    pub name: String,
}

pub struct GlFeature {
    pub api: Api,
    pub version: f32,
    pub gl_remove: Vec<GlRequire>,
    pub gl_require: Vec<GlRequire>,
}

pub struct GlRequire {
    pub gl_profile: Option<GlProfile>,
    pub api: Option<Api>,
    pub gl_enums: Vec<String>,
    pub gl_commands: Vec<String>,
}

pub struct GlExtension {}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("")]
    InvalidDocument,
    #[error("")]
    Profile(#[from] GlProfileFromStrError),
    #[error("")]
    Api(#[from] ApiFromStrError),
    #[error("Failed to parse registry file")]
    Xml(#[from] roxmltree::Error),
}

impl GlRegistry {
    pub fn parse(xml: &str) -> Result<Self, ParseError> {
        let document = Document::parse(xml)?;

        let mut gl_enums = Vec::new();
        let mut gl_commands = Vec::new();
        let mut gl_features = Vec::new();
        let mut gl_extensions = Vec::new();

        for node in document
            .root()
            .first_child()
            .ok_or(ParseError::InvalidDocument)?
            .children()
        {
            if node.is_element() {
                match node.tag_name().name() {
                    "enums" => {
                        if let Some(group) = node.attribute("group") {
                            // TODO: For some reasone nvidia used negative values here, no clue way. I'm fixing this another time.
                            if group == "TransformFeedbackTokenNV" {
                                continue;
                            }
                        }

                        let bitmask = if let Some(t) = node.attribute("type") {
                            t == "bitmask"
                        } else {
                            false
                        };

                        for gl_enum in node.children() {
                            if gl_enum.tag_name().name() == "enum" {
                                let name = gl_enum.attribute("name").unwrap().to_string();
                                let value = gl_enum.attribute("value").unwrap().to_string();
                                let group = gl_enum.attribute("group").map(|s| s.to_string());

                                gl_enums.push(GlEnum {
                                    name,
                                    value,
                                    bitmask,
                                    group,
                                });
                            }
                        }
                    }
                    "commands" => {
                        for gl_command in node.children() {
                            if gl_command.tag_name().name() == "command" {
                                let mut name = None;
                                let mut gl_params = Vec::new();
                                let mut return_type = None;

                                for command_attr in gl_command.children() {
                                    match command_attr.tag_name().name() {
                                        "proto" => {
                                            name = Some(
                                                command_attr
                                                    .children()
                                                    .find(|node| node.tag_name().name() == "name")
                                                    .unwrap()
                                                    .text()
                                                    .unwrap()
                                                    .to_string(),
                                            );

                                            let mut gl_type = if let Some(ptype) = command_attr
                                                .children()
                                                .find(|node| node.tag_name().name() == "ptype")
                                            {
                                                ptype.text().unwrap().to_string()
                                            } else {
                                                "".to_string()
                                            };

                                            if let Some(text) = command_attr.text() {
                                                if let "const" = text.trim() {
                                                    gl_type = format!("*const {}", gl_type);
                                                }
                                            }

                                            if !gl_type.is_empty() {
                                                return_type = Some(format!("->{}", gl_type));
                                            } else {
                                                return_type = Some(gl_type)
                                            }
                                        }
                                        "param" => {
                                            let mut name = command_attr
                                                .children()
                                                .find(|node| node.tag_name().name() == "name")
                                                .unwrap()
                                                .text()
                                                .unwrap()
                                                .to_string();

                                            if KEYWORDS.contains(&name.as_str()) {
                                                name = format!("r#{}", name)
                                            }

                                            let gl_type = if let Some(node) = command_attr
                                                .children()
                                                .find(|node| node.tag_name().name() == "ptype")
                                            {
                                                let mut gl_type = match node.text().unwrap().trim()
                                                {
                                                    "struct _cl_context" => "*mut _cl_context",
                                                    "struct _cl_event" => "*mut _cl_event",
                                                    gl_type => gl_type,
                                                }
                                                .to_string();

                                                if let Some(tail) = node.tail() {
                                                    if tail.trim() == "*" {
                                                        if let Some(text) = command_attr.text() {
                                                            if let "const" = text.trim() {
                                                                gl_type =
                                                                    format!("*const {}", gl_type);
                                                            }
                                                        } else {
                                                            gl_type = format!("*mut {}", gl_type);
                                                        }
                                                    } else if tail.trim() == "*const*" {
                                                        gl_type =
                                                            format!("*const *const {}", gl_type);
                                                    }
                                                }

                                                gl_type
                                            } else {
                                                match command_attr.text().unwrap().trim() {
                                                    "const void *" => "*const c_void",
                                                    "const void **" | "const void *const*" => {
                                                        "*const *const c_void"
                                                    }
                                                    "void *" => "*mut c_void",
                                                    "void **" => "*mut *mut c_void",
                                                    text => panic!(
                                                        "Couldn't find a valid type\n {}",
                                                        text
                                                    ),
                                                }
                                                .to_string()
                                            };

                                            gl_params.push(GlParam { name, gl_type })
                                        }
                                        "alias" => {}
                                        "glx" => {}
                                        "vecequiv" => {}
                                        _ => {
                                            // dbg!(name);
                                        }
                                    }
                                }

                                gl_commands.push(GlCommand {
                                    name: name.unwrap(),
                                    gl_params,
                                    return_type: return_type.unwrap(),
                                });
                            }
                        }
                    }
                    "feature" => {
                        let api = node.attribute("api").unwrap().parse()?;
                        let version = node.attribute("number").unwrap().parse().unwrap();
                        let mut gl_require = Vec::new();
                        let mut gl_remove = Vec::new();

                        for gl_feature in node.children() {
                            match gl_feature.tag_name().name() {
                                "require" => {
                                    let mut gl_enums = Vec::new();
                                    let mut gl_commands = Vec::new();
                                    let api =
                                        gl_feature.attribute("api").map(|api| api.parse().unwrap());
                                    let gl_profile = gl_feature
                                        .attribute("profile")
                                        .map(|profile| profile.parse().unwrap());

                                    for gl_require in gl_feature.children() {
                                        match gl_require.tag_name().name() {
                                            "enum" => {
                                                let gl_enum = gl_require.attribute("name").unwrap();
                                                gl_enums.push(gl_enum.to_string());
                                            }
                                            "command" => {
                                                let gl_command =
                                                    gl_require.attribute("name").unwrap();
                                                gl_commands.push(gl_command.to_string())
                                            }
                                            "type" => {}
                                            name => {
                                                if !name.is_empty() {
                                                    panic!("Unknown req {name}")
                                                }
                                            }
                                        }
                                    }

                                    gl_require.push(GlRequire {
                                        gl_enums,
                                        gl_commands,
                                        gl_profile,
                                        api,
                                    })
                                }
                                "remove" => {
                                    let mut gl_enums = Vec::new();
                                    let mut gl_commands = Vec::new();
                                    let api =
                                        gl_feature.attribute("api").map(|api| api.parse().unwrap());
                                    let gl_profile = gl_feature
                                        .attribute("profile")
                                        .map(|profile| profile.parse().unwrap());

                                    for gl_require in gl_feature.children() {
                                        match gl_require.tag_name().name() {
                                            "enum" => {
                                                let gl_enum = gl_require.attribute("name").unwrap();
                                                gl_enums.push(gl_enum.to_string());
                                            }
                                            "command" => {
                                                let gl_command =
                                                    gl_require.attribute("name").unwrap();
                                                gl_commands.push(gl_command.to_string())
                                            }
                                            "type" => {}
                                            name => {
                                                if !name.is_empty() {
                                                    panic!("Unknown req {name}")
                                                }
                                            }
                                        }
                                    }

                                    gl_remove.push(GlRequire {
                                        gl_enums,
                                        gl_commands,
                                        gl_profile,
                                        api,
                                    })
                                }
                                name => {
                                    if !name.is_empty() {
                                        panic!("Unknown req {name}")
                                    }
                                }
                            }
                        }

                        gl_features.push(GlFeature {
                            api,
                            version,
                            gl_require,
                            gl_remove,
                        })
                    }
                    _ => {}
                }
            }
        }

        Ok(Self {
            gl_enums,
            gl_commands,
            gl_features,
            gl_extensions,
        })
    }

    // TODO: This code is horribily inefficient, it literaly takes seconds to execute in debug mode. I'll fix it as some point but it works for now.
    pub fn reduce(&mut self, api: Api, version: f32, profile: GlProfile) {
        self.gl_features
            .retain(|gl_feature| gl_feature.api == api && gl_feature.version <= version);

        let mut required_enums: HashSet<&String> = HashSet::new();
        let mut required_commands: HashSet<&String> = HashSet::new();

        for gl_feature in &self.gl_features {
            for gl_require in &gl_feature.gl_require {
                if (gl_require.gl_profile.is_none() || gl_require.gl_profile == Some(profile))
                    && (gl_require.api.is_none() || gl_require.api == Some(api))
                {
                    required_enums.extend(&gl_require.gl_enums);
                    required_commands.extend(&gl_require.gl_commands);
                }
            }

            for gl_remove in &gl_feature.gl_remove {
                if (gl_remove.gl_profile.is_none() || gl_remove.gl_profile == Some(profile))
                    && (gl_remove.api.is_none() || gl_remove.api == Some(api))
                {
                    required_enums
                        .retain(|required_enum| !gl_remove.gl_enums.contains(required_enum));
                    required_commands.retain(|required_command| {
                        !gl_remove.gl_commands.contains(required_command)
                    });
                }
            }
        }

        self.gl_commands
            .retain(|gl_command| required_commands.contains(&&gl_command.name));

        self.gl_enums
            .retain(|gl_enum| required_enums.contains(&&gl_enum.name));
    }
}
