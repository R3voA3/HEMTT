use std::{
    env::temp_dir,
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};

use hemtt_common::project::ProjectConfig;
use hemtt_common::workspace::{Workspace, WorkspacePath};

use crate::{addons::Addon, error::Error};

#[derive(Debug, Clone)]
pub struct Context {
    config: ProjectConfig,
    folder: String,
    addons: Vec<Addon>,
    workspace: WorkspacePath,
    project_folder: PathBuf,
    hemtt_folder: PathBuf,
    out_folder: PathBuf,
    tmp: PathBuf,
}

impl Context {
    /// Create a new context
    ///
    /// # Errors
    /// [`Error::ConfigNotFound`] if the project.toml is not found
    /// [`Error::Io`] if the temporary folder fails to be created
    /// [`Error::Git`] if the git hash is invalid
    /// [`Error::Version`] if the version is invalid
    ///
    /// # Panics
    /// If the project folder is not a valid [`OsStr`] (UTF-8)
    pub fn new(root: PathBuf, folder: &str) -> Result<Self, Error> {
        let config = {
            let path = root.join(".hemtt").join("project.toml");
            if !path.exists() {
                return Err(Error::ConfigNotFound);
            }
            ProjectConfig::from_file(&path)?
        };
        let tmp = {
            let mut tmp = temp_dir().join("hemtt");
            // on linux add the user to the path for multiple users
            if !cfg!(target_os = "windows") {
                tmp = tmp.join(whoami::username());
            }
            tmp
        }
        .join(
            root.components()
                .skip(2)
                .collect::<PathBuf>()
                .to_str()
                .expect("valid utf-8")
                .replace(['\\', '/'], "_"),
        );
        trace!("using temporary folder: {:?}", tmp.display());
        let hemtt_folder = root.join(".hemtt");
        trace!("using project folder: {:?}", root.display());
        let out_folder = root.join(".hemttout");
        trace!("using out folder: {:?}", out_folder.display());
        create_dir_all(&out_folder)?;
        let build_folder = out_folder.join(folder);
        trace!("using build folder: {:?}", build_folder.display());
        if build_folder.exists() {
            remove_dir_all(&build_folder)?;
        }
        create_dir_all(&build_folder)?;
        let workspace = {
            let mut builder = Workspace::builder().physical(&root);
            if cfg!(target_os = "windows") {
                builder = builder.physical(&tmp.join("output"));
            }
            let include = root.join("include");
            if include.is_dir() {
                builder = builder.physical(&include);
            }
            builder.memory().finish(Some(config.clone()))?
        };
        {
            let version = config.version().get(workspace.vfs());
            if let Err(hemtt_common::project::Error::Git(_)) = version {
                error!("Failed to find a git repository with at least one commit, if you are not using git add the following to your project.toml");
                println!("\n[version]\ngit_hash = 0\n");
                std::process::exit(1);
            };
            info!("Config loaded for {} {}", config.name(), version?);
        }
        Ok(Self {
            config,
            folder: folder.to_owned(),
            workspace,
            project_folder: root,
            hemtt_folder,
            out_folder: build_folder,
            addons: Addon::scan()?,
            tmp,
        })
    }

    #[must_use]
    pub fn filter<F>(self, mut filter: F) -> Self
    where
        F: FnMut(&Addon, &ProjectConfig) -> bool,
    {
        let config = self.config.clone();
        Self {
            addons: self
                .addons
                .into_iter()
                .filter(|a| filter(a, &config))
                .collect(),
            ..self
        }
    }

    #[must_use]
    pub const fn config(&self) -> &ProjectConfig {
        &self.config
    }

    #[must_use]
    pub fn folder(&self) -> &str {
        &self.folder
    }

    #[must_use]
    pub fn addons(&self) -> &[Addon] {
        &self.addons
    }

    #[must_use]
    pub fn addon(&self, name: &str) -> Option<&Addon> {
        self.addons.iter().find(|a| a.name() == name)
    }

    #[must_use]
    pub const fn workspace(&self) -> &WorkspacePath {
        &self.workspace
    }

    #[must_use]
    /// The project folder
    pub const fn project_folder(&self) -> &PathBuf {
        &self.project_folder
    }

    #[must_use]
    /// The .hemtt folder
    pub const fn hemtt_folder(&self) -> &PathBuf {
        &self.hemtt_folder
    }

    #[must_use]
    /// The .hemttout folder
    pub const fn out_folder(&self) -> &PathBuf {
        &self.out_folder
    }

    #[must_use]
    /// %temp%/hemtt/project
    pub const fn tmp(&self) -> &PathBuf {
        &self.tmp
    }
}
