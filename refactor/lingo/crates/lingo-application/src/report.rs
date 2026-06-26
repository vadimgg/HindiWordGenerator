use lingo_domain::{BatchId, RawDocumentId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandHint(String);

impl CommandHint {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NextAction {
    Import { raw: Option<RawDocumentId> },
    Build { batch: BatchId },
    Check { batch: Option<BatchId> },
    Audio { batch: Option<BatchId> },
    Package,
    Export,
    None,
}

impl NextAction {
    pub fn command_hint(&self) -> Option<CommandHint> {
        match self {
            Self::Import { raw: Some(raw) } => Some(CommandHint::new(format!(
                "lingo import raw/{}.txt",
                raw.as_str()
            ))),
            Self::Import { raw: None } => Some(CommandHint::new("lingo import")),
            Self::Build { batch } => Some(CommandHint::new(format!(
                "lingo build --batch {}",
                batch.as_str()
            ))),
            Self::Check { batch: Some(batch) } => Some(CommandHint::new(format!(
                "lingo check --batch {}",
                batch.as_str()
            ))),
            Self::Check { batch: None } => Some(CommandHint::new("lingo check")),
            Self::Audio { batch: Some(batch) } => Some(CommandHint::new(format!(
                "lingo audio --batch {}",
                batch.as_str()
            ))),
            Self::Audio { batch: None } => Some(CommandHint::new("lingo audio")),
            Self::Package => Some(CommandHint::new("lingo package")),
            Self::Export => Some(CommandHint::new("lingo export")),
            Self::None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChangeCounts {
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
}

#[cfg(test)]
mod tests {
    use super::NextAction;
    use lingo_domain::BatchId;

    #[test]
    fn maps_next_action_once() {
        let action = NextAction::Build {
            batch: BatchId::parse("chapter-01").unwrap(),
        };
        assert_eq!(
            action.command_hint().unwrap().as_str(),
            "lingo build --batch chapter-01"
        );
    }
}
