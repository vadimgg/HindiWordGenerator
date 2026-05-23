use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRoot {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRootError {
    start: PathBuf,
}

impl ProjectRootError {
    pub(crate) fn new(start: impl Into<PathBuf>) -> Self {
        Self {
            start: start.into(),
        }
    }
}

impl ProjectRoot {
    pub fn discover_from(start: impl AsRef<Path>) -> Result<Self, ProjectRootError> {
        let start = start.as_ref();
        let mut current = if start.is_file() {
            start.parent().unwrap_or(start).to_path_buf()
        } else {
            start.to_path_buf()
        };

        loop {
            if is_project_root(&current) {
                return Ok(Self { path: current });
            }

            if !current.pop() {
                return Err(ProjectRootError::new(start.to_path_buf()));
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn join(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.path.join(relative)
    }
}

impl std::fmt::Display for ProjectRootError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Hindi workspace not found\n\nStarted from {}\n\nRun this command from a Hindi workspace or initialize the current folder:\n  hindi init",
            self.start.display()
        )
    }
}

fn is_project_root(path: &Path) -> bool {
    path.join("hindi.toml").is_file()
        || path.join("input/sentences").is_dir()
        || (path.join("docs/DESIGN.md").is_file()
            && path.join("docs/ROADMAP.md").is_file()
            && path.join("input").is_dir()
            && path.join("output").is_dir())
}

#[cfg(test)]
mod tests {
    use super::ProjectRoot;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn discovers_project_root_from_child_directory() {
        let root = temp_path("hindi-root");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("input")).unwrap();
        fs::create_dir_all(root.join("output")).unwrap();
        fs::create_dir_all(root.join("nested/child")).unwrap();
        fs::write(root.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "").unwrap();

        let discovered = ProjectRoot::discover_from(root.join("nested/child")).unwrap();

        assert_eq!(discovered.path(), root.as_path());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discovers_workspace_from_hindi_toml() {
        let root = temp_path("hindi-workspace");
        fs::create_dir_all(root.join("nested/child")).unwrap();
        fs::write(root.join("hindi.toml"), "").unwrap();

        let discovered = ProjectRoot::discover_from(root.join("nested/child")).unwrap();

        assert_eq!(discovered.path(), root.as_path());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discovers_workspace_from_input_sentences() {
        let root = temp_path("hindi-input-sentences");
        fs::create_dir_all(root.join("input/sentences")).unwrap();

        let discovered = ProjectRoot::discover_from(root.join("input/sentences")).unwrap();

        assert_eq!(discovered.path(), root.as_path());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fails_when_project_root_is_missing() {
        let root = temp_path("not-hindi-root");
        fs::create_dir_all(root.join("nested")).unwrap();

        let error = ProjectRoot::discover_from(root.join("nested")).unwrap_err();

        assert!(error.to_string().contains("Hindi workspace not found"));
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
    }
}
