use crate::error::ArtifactError;
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ArtifactPath(String);

impl ArtifactPath {
    pub fn parse(raw: impl Into<String>) -> Result<Self, ArtifactError> {
        let raw = raw.into().replace('\\', "/");
        let valid = !raw.is_empty()
            && !raw.starts_with('/')
            && !raw.contains(':')
            && raw.split('/').all(|part| {
                !part.is_empty() && part != "." && part != ".." && !part.contains('\0')
            });
        if !valid {
            return Err(ArtifactError::UnsafePath(raw));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn join_to(&self, root: &Path) -> std::path::PathBuf {
        root.join(&self.0)
    }
}

impl fmt::Display for ArtifactPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactFile {
    pub path: ArtifactPath,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PublicationPlan {
    files: Vec<ArtifactFile>,
}

impl PublicationPlan {
    pub fn try_new(mut files: Vec<ArtifactFile>) -> Result<Self, ArtifactError> {
        files.sort_by(|left, right| left.path.cmp(&right.path));
        let mut seen = BTreeSet::new();
        for file in &files {
            if !seen.insert(file.path.clone()) {
                return Err(ArtifactError::DuplicatePath(file.path.to_string()));
            }
        }
        Ok(Self { files })
    }

    pub fn files(&self) -> &[ArtifactFile] {
        &self.files
    }

    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|file| file.bytes.len() as u64).sum()
    }
}
