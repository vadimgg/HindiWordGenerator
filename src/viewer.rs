use crate::project::{ProjectRoot, ProjectRootError};
use std::io;
use std::path::PathBuf;
use std::process::Command;

const VIEWER_DIR: &str = "viewer";
const VIEWER_URL: &str = "http://localhost:4321";

#[derive(Debug)]
pub enum ViewerError {
    Project(ProjectRootError),
    Io { path: PathBuf, source: io::Error },
    MissingApp(PathBuf),
    Exited(i32),
}

impl std::fmt::Display for ViewerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewerError::Project(error) => write!(formatter, "{error}"),
            ViewerError::Io { path, source } => {
                write!(
                    formatter,
                    "Could not run viewer from {}\n\n{source}",
                    path.display()
                )
            }
            ViewerError::MissingApp(path) => {
                write!(formatter, "Viewer app not found: {}", path.display())
            }
            ViewerError::Exited(code) => write!(formatter, "Viewer exited with status {code}."),
        }
    }
}

impl From<ProjectRootError> for ViewerError {
    fn from(error: ProjectRootError) -> Self {
        ViewerError::Project(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewerCommand {
    pub app_dir: PathBuf,
    pub url: String,
    pub command: Vec<String>,
}

impl ViewerCommand {
    pub fn render(&self) -> String {
        format!(
            "Viewer\n\n  app        {}\n  url        {}\n  command    {}\n\nPress Ctrl-C to stop the viewer.",
            self.app_dir.display(),
            self.url,
            self.command.join(" ")
        )
    }
}

pub fn command_from_current_dir() -> Result<ViewerCommand, ViewerError> {
    let root = ProjectRoot::discover_from(current_dir()?)?;
    command_for(&root)
}

pub fn run_from_current_dir() -> Result<(), ViewerError> {
    let command = command_from_current_dir()?;
    println!("{}", command.render());
    let status = Command::new(&command.command[0])
        .args(&command.command[1..])
        .current_dir(&command.app_dir)
        .status()
        .map_err(|source| ViewerError::Io {
            path: command.app_dir.clone(),
            source,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(ViewerError::Exited(status.code().unwrap_or(1)))
    }
}

pub fn command_for(root: &ProjectRoot) -> Result<ViewerCommand, ViewerError> {
    let app_dir = root.join(VIEWER_DIR);
    let package = app_dir.join("package.json");
    if !package.is_file() {
        return Err(ViewerError::MissingApp(package));
    }
    Ok(ViewerCommand {
        app_dir,
        url: VIEWER_URL.to_string(),
        command: vec!["npm".to_string(), "run".to_string(), "dev".to_string()],
    })
}

fn current_dir() -> Result<PathBuf, ViewerError> {
    std::env::current_dir().map_err(|source| ViewerError::Io {
        path: PathBuf::from("."),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::command_for;
    use crate::project::ProjectRoot;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_viewer_command_from_project_root() {
        let root = fixture_root(true);
        let project = ProjectRoot::discover_from(&root).unwrap();

        let command = command_for(&project).unwrap();

        assert_eq!(command.url, "http://localhost:4321");
        assert_eq!(command.command, ["npm", "run", "dev"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_missing_viewer_app() {
        let root = fixture_root(false);
        let project = ProjectRoot::discover_from(&root).unwrap();

        let error = command_for(&project).unwrap_err();

        assert!(error.to_string().contains("viewer/package.json"));
        fs::remove_dir_all(root).unwrap();
    }

    fn fixture_root(with_viewer: bool) -> PathBuf {
        let root = temp_path("hindi-viewer");
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("input")).unwrap();
        fs::create_dir_all(root.join("output")).unwrap();
        fs::write(root.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "").unwrap();
        if with_viewer {
            fs::create_dir_all(root.join("viewer")).unwrap();
            fs::write(root.join("viewer/package.json"), "{}").unwrap();
        }
        root
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
    }
}
