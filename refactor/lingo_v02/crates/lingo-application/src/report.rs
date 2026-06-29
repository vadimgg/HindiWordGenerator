#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandHint(String);

impl CommandHint {
    pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NextAction {
    Extract,
    Enrich,
    Audio,
    Package,
    Export,
    None,
}

impl NextAction {
    pub fn command_hint(&self) -> Option<CommandHint> {
        match self {
            Self::Extract => Some(CommandHint::new("lingo extract <RAW> --print")),
            Self::Enrich => Some(CommandHint::new("lingo enrich --limit 20 --print")),
            Self::Audio => Some(CommandHint::new("lingo audio --missing")),
            Self::Package => Some(CommandHint::new("lingo package --dest packages/sentences")),
            Self::Export => Some(CommandHint::new("lingo export --dest exports/lingo.apkg")),
            Self::None => None,
        }
    }
}
