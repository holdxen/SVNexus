use std::path::{Path, PathBuf};

use crate::error;


pub struct TemporaryDirectory {
    cache_dir: PathBuf,
    config_dir: PathBuf,
    config_local_dir: PathBuf,
    data_dir: PathBuf,
    data_local_dir: PathBuf,
    preference_dir: PathBuf,
    runtime_dir: Option<PathBuf>,
    state_dir: Option<PathBuf>,
}

impl TemporaryDirectory {
    fn create() -> error::Result<Self> {
        Ok(TemporaryDirectory {
            cache_dir: tempfile::tempdir()?.keep(),
            config_dir: tempfile::tempdir()?.keep(),
            config_local_dir: tempfile::tempdir()?.keep(),
            data_dir: tempfile::tempdir()?.keep(),
            data_local_dir: tempfile::tempdir()?.keep(),
            preference_dir: tempfile::tempdir()?.keep(),
            runtime_dir: None,
            state_dir: None,
        })
    }
}

impl ProjectDirectory for TemporaryDirectory {
    fn cache_directory(&self) -> &Path {
        &self.cache_dir
    }

    fn config_directory(&self) -> &Path {
        &self.config_dir
    }

    fn config_local_directory(&self) -> &Path {
        &self.config_local_dir
    }

    fn data_directory(&self) -> &Path {
        &self.data_dir
    }

    fn data_local_directory(&self) -> &Path {
        &self.data_local_dir
    }

    fn preference_directory(&self) -> &Path {
        &self.preference_dir
    }

    fn runtime_directory(&self) -> Option<&Path> {
        self.runtime_dir.as_deref()
    }

    fn state_directory(&self) -> Option<&Path> {
        self.state_dir.as_deref()
    }
}

pub trait ProjectDirectory: Send + Sync + 'static {
    fn cache_directory(&self) -> &Path;
    fn config_directory(&self) -> &Path;
    fn config_local_directory(&self) -> &Path;
    fn data_directory(&self) -> &Path;
    fn data_local_directory(&self) -> &Path;
    fn preference_directory(&self) -> &Path;
    fn runtime_directory(&self) -> Option<&Path>;
    fn state_directory(&self) -> Option<&Path>;
}

impl ProjectDirectory for directories::ProjectDirs {
    fn cache_directory(&self) -> &Path {
        self.cache_dir()
    }

    fn config_directory(&self) -> &Path {
        self.config_dir()
    }

    fn config_local_directory(&self) -> &Path {
        self.config_local_dir()
    }

    fn data_directory(&self) -> &Path {
        self.data_dir()
    }

    fn data_local_directory(&self) -> &Path {
        self.data_local_dir()
    }

    fn preference_directory(&self) -> &Path {
        self.preference_dir()
    }

    fn runtime_directory(&self) -> Option<&Path> {
        self.runtime_dir()
    }

    fn state_directory(&self) -> Option<&Path> {
        self.state_dir()
    }
}


pub fn project() -> error::Result<&'static Box<dyn ProjectDirectory>> {

    static PROJECT: once_cell::sync::OnceCell<Box<dyn ProjectDirectory>> = once_cell::sync::OnceCell::new();

    PROJECT.get_or_try_init(|| {
        let project: Box<dyn ProjectDirectory> = if let Some(project) = directories::ProjectDirs::from("io.github", "holdxen", "SVNexus") {
            std::fs::create_dir_all(project.cache_directory())?;
            std::fs::create_dir_all(project.config_directory())?;
            std::fs::create_dir_all(project.config_local_directory())?;
            std::fs::create_dir_all(project.data_directory())?;
            std::fs::create_dir_all(project.data_local_directory())?;
            std::fs::create_dir_all(project.preference_directory())?;
            if let Some(runtime_dir) = project.runtime_directory() {
                std::fs::create_dir_all(runtime_dir)?;
            }
            if let Some(state_dir) = project.state_directory() {
                std::fs::create_dir_all(state_dir)?;
            }
            Box::new(project)
        } else {
            Box::new(TemporaryDirectory::create()?)
        };



        error::ok(project)
    })

}

pub struct GlobalConfig {
    log: String,
    engine: EngineConfig,
}

pub struct EngineConfig {
    username: Option<String>,
    password: Option<String>,
}

// fn app() {
//     let project = directories::ProjectDirs::from("io.github", "holdxen", "SVNexus").unwrap();
// }