use crate::project::{ProjectRoot, ProjectRootError};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

const OLLAMA_VERSION_ENDPOINT: &str = "http://localhost:11434/api/version";

pub trait OllamaChecker {
    fn check(&self) -> OllamaStatus;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OllamaStatus {
    Ok,
    Unreachable,
}

impl OllamaStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Unreachable => "unreachable",
        }
    }

    fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

pub struct HttpOllamaChecker;

impl OllamaChecker for HttpOllamaChecker {
    fn check(&self) -> OllamaStatus {
        match check_ollama_version_endpoint() {
            true => OllamaStatus::Ok,
            false => OllamaStatus::Unreachable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    root: ProjectRoot,
    config: Check,
    data: Vec<Check>,
    prompts: Vec<Check>,
    ollama: OllamaStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Check {
    name: &'static str,
    state: CheckState,
    path: &'static str,
    required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckState {
    Ok,
    Missing,
}

impl CheckState {
    fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
        }
    }
}

impl DoctorReport {
    pub fn collect(root: ProjectRoot, ollama_checker: &dyn OllamaChecker) -> Self {
        let config = check_file(&root, "config", "hindi.toml", false);
        let data = vec![
            check_dir(&root, "input", "input/", true),
            check_dir(&root, "sentences", "input/sentences/", true),
            check_dir(&root, "words", "input/words/", true),
            check_dir(&root, "output", "output/", true),
            check_dir(&root, "audio", "audio/", true),
        ];
        let prompts = vec![
            check_builtin("sentences", "built-in staged prompts", true),
            check_file(
                &root,
                "legacy",
                "generation_prompt_sentences_enrichment.txt",
                false,
            ),
            check_file(&root, "python", "generation_prompt_sentences.txt", false),
        ];

        Self {
            root,
            config,
            data,
            prompts,
            ollama: ollama_checker.check(),
        }
    }

    pub fn required_checks_passed(&self) -> bool {
        self.data
            .iter()
            .chain(self.prompts.iter())
            .filter(|check| check.required)
            .all(|check| check.state == CheckState::Ok)
            && self.ollama.is_ok()
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("Hindi Word Generator\n\n");
        output.push_str("Project\n");
        output.push_str(&format!("  root       {}\n", self.root.path().display()));
        output.push_str(&format!(
            "  config     {:<8} {}\n\n",
            self.config.state.label(),
            self.config.path
        ));

        output.push_str("Data\n");
        for check in &self.data {
            output.push_str(&format!(
                "  {:<9} {:<8} {}\n",
                check.name,
                check.state.label(),
                check.path
            ));
        }
        output.push('\n');

        output.push_str("Prompts\n");
        for check in &self.prompts {
            output.push_str(&format!(
                "  {:<9} {:<8} {}\n",
                check.name,
                check.state.label(),
                check.path
            ));
        }
        output.push('\n');

        output.push_str("Ollama\n");
        output.push_str(&format!(
            "  service    {:<8} {}\n",
            self.ollama.label(),
            OLLAMA_VERSION_ENDPOINT
        ));
        output.push_str("  model      not checked in M1\n\n");

        if !self.ollama.is_ok() {
            output.push_str("Ollama is not reachable.\n\n");
            output.push_str("Start it, then rerun:\n");
            output.push_str("  hindi doctor\n\n");
        }

        output.push_str("Next\n");
        output.push_str("  cargo run -- sentences plan --max-batches 1");
        output
    }
}

pub fn run_from_current_dir(
    ollama_checker: &dyn OllamaChecker,
) -> Result<DoctorReport, ProjectRootError> {
    let current_dir =
        std::env::current_dir().map_err(|_| ProjectRootError::new("<current directory>"))?;
    run_from(current_dir, ollama_checker)
}

pub fn run_from(
    start: impl AsRef<Path>,
    ollama_checker: &dyn OllamaChecker,
) -> Result<DoctorReport, ProjectRootError> {
    let root = ProjectRoot::discover_from(start)?;
    Ok(DoctorReport::collect(root, ollama_checker))
}

fn check_dir(root: &ProjectRoot, name: &'static str, path: &'static str, required: bool) -> Check {
    check_path(root, name, path, required, PathKind::Directory)
}

fn check_file(root: &ProjectRoot, name: &'static str, path: &'static str, required: bool) -> Check {
    check_path(root, name, path, required, PathKind::File)
}

fn check_builtin(name: &'static str, path: &'static str, required: bool) -> Check {
    Check {
        name,
        state: CheckState::Ok,
        path,
        required,
    }
}

fn check_path(
    root: &ProjectRoot,
    name: &'static str,
    path: &'static str,
    required: bool,
    kind: PathKind,
) -> Check {
    let absolute = root.join(path);
    let exists = match kind {
        PathKind::Directory => absolute.is_dir(),
        PathKind::File => absolute.is_file(),
    };

    Check {
        name,
        state: if exists {
            CheckState::Ok
        } else {
            CheckState::Missing
        },
        path,
        required,
    }
}

#[derive(Debug, Clone, Copy)]
enum PathKind {
    Directory,
    File,
}

fn check_ollama_version_endpoint() -> bool {
    let addrs = match ("localhost", 11434).to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(_) => return false,
    };

    for addr in addrs {
        let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(400)) else {
            continue;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_millis(400)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(400)));

        let request =
            b"GET /api/version HTTP/1.1\r\nHost: localhost:11434\r\nConnection: close\r\n\r\n";
        if stream.write_all(request).is_err() {
            continue;
        }

        let mut response = String::new();
        if stream.read_to_string(&mut response).is_err() {
            continue;
        }

        if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{run_from, DoctorReport, OllamaChecker, OllamaStatus};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct FakeOllamaChecker {
        status: OllamaStatus,
    }

    impl OllamaChecker for FakeOllamaChecker {
        fn check(&self) -> OllamaStatus {
            self.status
        }
    }

    #[test]
    fn missing_optional_config_does_not_fail() {
        let root = create_project();
        let report = collect(&root, OllamaStatus::Ok);

        assert!(report.render().contains("config     missing  hindi.toml"));
        assert!(report.required_checks_passed());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_prompt_files_are_optional() {
        let root = create_project();
        fs::remove_file(root.join("generation_prompt_sentences_enrichment.txt")).unwrap();
        fs::remove_file(root.join("generation_prompt_sentences.txt")).unwrap();
        let report = collect(&root, OllamaStatus::Ok);

        assert!(report.required_checks_passed());
        assert!(report
            .render()
            .contains("sentences ok       built-in staged prompts"));
        assert!(report
            .render()
            .contains("legacy    missing  generation_prompt_sentences_enrichment.txt"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ollama_unreachable_fails_with_recovery_text() {
        let root = create_project();
        let report = collect(&root, OllamaStatus::Unreachable);

        assert!(!report.required_checks_passed());
        assert!(report.render().contains("Ollama is not reachable."));
        assert!(report.render().contains("hindi doctor"));
        fs::remove_dir_all(root).unwrap();
    }

    fn collect(root: &PathBuf, status: OllamaStatus) -> DoctorReport {
        run_from(root, &FakeOllamaChecker { status }).unwrap()
    }

    fn create_project() -> PathBuf {
        let root = temp_path("hindi-doctor");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("input/sentences")).unwrap();
        fs::create_dir_all(root.join("input/words")).unwrap();
        fs::create_dir_all(root.join("output")).unwrap();
        fs::create_dir_all(root.join("audio")).unwrap();
        fs::write(root.join("docs/DESIGN.md"), "").unwrap();
        fs::write(root.join("docs/ROADMAP.md"), "").unwrap();
        fs::write(root.join("generation_prompt_sentences_enrichment.txt"), "").unwrap();
        fs::write(root.join("generation_prompt_sentences.txt"), "").unwrap();
        root
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("{label}-{}-{nanos}-{count}", std::process::id()))
    }
}
