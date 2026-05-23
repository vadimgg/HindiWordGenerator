use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const CONFIG_PATH: &str = "hindi.toml";
const DEFAULT_CONFIG: &str = r#"[models]
sentence_generation = "ollama:translategemma:12b"
"#;

const DIRECTORIES: [&str; 8] = [
    "input/sentences",
    "input/words",
    "output/sentences",
    "output/words",
    "audio/sentences",
    "audio/words",
    "runs",
    "exports",
];

#[derive(Debug)]
pub enum InitError {
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for InitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::Io { path, source } => {
                write!(
                    formatter,
                    "Could not initialize {}\n\n{source}",
                    path.display()
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitReport {
    root: PathBuf,
    config_created: bool,
    created_dirs: Vec<PathBuf>,
    existing_dirs: Vec<PathBuf>,
}

impl InitReport {
    pub fn render(&self) -> String {
        let mut output = String::from("Hindi Workspace Init\n\n");
        output.push_str(&format!("  root             {}\n", self.root.display()));
        output.push_str(&format!(
            "  config           {}\n",
            if self.config_created {
                "created hindi.toml"
            } else {
                "kept existing hindi.toml"
            }
        ));
        output.push_str(&format!("  created dirs     {}\n", self.created_dirs.len()));
        output.push_str(&format!(
            "  existing dirs    {}\n",
            self.existing_dirs.len()
        ));

        if !self.created_dirs.is_empty() {
            output.push_str("\nCreated\n");
            for path in &self.created_dirs {
                output.push_str(&format!("  {}\n", path.display()));
            }
        }

        output.push_str("\nLayout\n");
        output.push_str("  input/sentences/   source sentence YAML\n");
        output.push_str("  output/sentences/  accepted generated JSON\n");
        output.push_str("  audio/sentences/   generated MP3 files\n");

        output.push_str("\nNext\n");
        output.push_str("  Add YAML files under input/sentences/\n");
        output.push_str("  hindi guide\n");
        output
    }
}

pub fn init_current_dir() -> Result<InitReport, InitError> {
    let root = std::env::current_dir().map_err(|source| InitError::Io {
        path: PathBuf::from("."),
        source,
    })?;
    init_at(root)
}

pub fn init_at(root: impl AsRef<Path>) -> Result<InitReport, InitError> {
    let root = root.as_ref().to_path_buf();
    fs::create_dir_all(&root).map_err(|source| InitError::Io {
        path: root.clone(),
        source,
    })?;

    let config_path = root.join(CONFIG_PATH);
    let config_created = if config_path.exists() {
        false
    } else {
        write_new_file(&config_path, DEFAULT_CONFIG.as_bytes())?;
        true
    };

    let mut created_dirs = Vec::new();
    let mut existing_dirs = Vec::new();
    for relative in DIRECTORIES {
        let path = root.join(relative);
        if path.is_dir() {
            existing_dirs.push(PathBuf::from(relative));
        } else {
            fs::create_dir_all(&path).map_err(|source| InitError::Io {
                path: path.clone(),
                source,
            })?;
            created_dirs.push(PathBuf::from(relative));
        }
    }

    Ok(InitReport {
        root,
        config_created,
        created_dirs,
        existing_dirs,
    })
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), InitError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| InitError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    file.write_all(bytes).map_err(|source| InitError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::init_at;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn creates_workspace_layout() {
        let root = temp_path("hindi-init");

        let report = init_at(&root).unwrap();

        assert!(report.config_created);
        assert!(root.join("hindi.toml").is_file());
        assert!(root.join("input/sentences").is_dir());
        assert!(root.join("output/sentences").is_dir());
        assert!(root.join("audio/sentences").is_dir());
        assert!(root.join("runs").is_dir());
        assert!(root.join("exports").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_existing_config() {
        let root = temp_path("hindi-init-existing");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("hindi.toml"), "# custom\n").unwrap();

        let report = init_at(&root).unwrap();

        assert!(!report.config_created);
        assert_eq!(
            fs::read_to_string(root.join("hindi.toml")).unwrap(),
            "# custom\n"
        );
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
