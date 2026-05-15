use std::io;
use std::path::Path;
use std::process::Command;

pub trait TtsBackend {
    fn synthesize_hindi(&self, text: &str, target: &Path) -> Result<(), TtsError>;
}

#[derive(Debug)]
pub enum TtsError {
    Io(io::Error),
    Failed { status: Option<i32>, stderr: String },
}

impl std::fmt::Display for TtsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TtsError::Io(error) => write!(formatter, "TTS backend failed to start: {error}"),
            TtsError::Failed { status, stderr } => {
                write!(
                    formatter,
                    "TTS backend failed with status {:?}: {}",
                    status,
                    stderr.trim()
                )
            }
        }
    }
}

pub struct UvGttsBackend;

impl TtsBackend for UvGttsBackend {
    fn synthesize_hindi(&self, text: &str, target: &Path) -> Result<(), TtsError> {
        let code = r#"from gtts import gTTS
import sys
gTTS(text=sys.argv[1], lang="hi", slow=False).save(sys.argv[2])
"#;
        let output = Command::new("uv")
            .args(["run", "--with", "gtts", "python", "-c", code, text])
            .arg(target)
            .output()
            .map_err(TtsError::Io)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(TtsError::Failed {
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }
}
