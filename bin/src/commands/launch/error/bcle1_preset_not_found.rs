use std::{path::Path, sync::Arc};

use hemtt_common::{
    reporting::{Code, Diagnostic},
    similar_values,
};

pub struct PresetNotFound {
    name: String,
    similar: Vec<String>,
}

impl Code for PresetNotFound {
    fn ident(&self) -> &'static str {
        "BCLE1"
    }

    fn message(&self) -> String {
        format!("Preset `{}` not found.", self.name)
    }

    fn help(&self) -> Option<String> {
        if self.similar.is_empty() {
            None
        } else {
            Some(format!("did you mean `{}`?", self.similar.join("`, `")))
        }
    }

    fn diagnostic(&self) -> Option<Diagnostic> {
        Some(Diagnostic::simple(self))
    }
}

impl PresetNotFound {
    pub fn code(name: String, path: &Path) -> Arc<dyn Code> {
        let presets = path.read_dir().map_or_else(
            |_| vec![],
            |files| {
                files
                    .filter_map(|x| {
                        x.ok().and_then(|x| {
                            if x.file_type().ok()?.is_file() {
                                x.file_name()
                                    .to_str()
                                    .map(|s| s.trim_end_matches(".html").to_string())
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<String>>()
            },
        );

        // let position = std::fs::read_to_string(".hemtt/project.toml")
        //     .map_or(None, |content| attempt_locate(&content, launch, &name));

        Arc::new(Self {
            similar: similar_values(
                &name,
                &presets
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<&str>>(),
            )
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
            name,
        })
    }
}
