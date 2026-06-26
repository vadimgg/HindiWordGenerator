use crate::BatchId;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PipelineStage {
    Raw,
    Source,
    Cards,
    Check,
    Audio,
    Package,
    Export,
}

impl PipelineStage {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Source => "source",
            Self::Cards => "cards",
            Self::Check => "check",
            Self::Audio => "audio",
            Self::Package => "package",
            Self::Export => "export",
        }
    }

    pub const fn ordered() -> &'static [Self] {
        &[
            Self::Raw,
            Self::Source,
            Self::Cards,
            Self::Check,
            Self::Audio,
            Self::Package,
            Self::Export,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckState {
    NotRun,
    Clean,
    Problems { errors: usize, warnings: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioCoverage {
    present: usize,
    total: usize,
}

impl AudioCoverage {
    pub const fn new(present: usize, total: usize) -> Self {
        Self {
            present: if present > total { total } else { present },
            total,
        }
    }

    pub const fn present(self) -> usize {
        self.present
    }

    pub const fn total(self) -> usize {
        self.total
    }

    pub const fn is_complete(self) -> bool {
        self.total > 0 && self.present == self.total
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchProgress {
    batch: BatchId,
    raw_present: bool,
    source_present: bool,
    cards_present: bool,
    check: CheckState,
    audio: AudioCoverage,
}

impl BatchProgress {
    pub const fn new(
        batch: BatchId,
        raw_present: bool,
        source_present: bool,
        cards_present: bool,
        check: CheckState,
        audio: AudioCoverage,
    ) -> Self {
        Self {
            batch,
            raw_present,
            source_present,
            cards_present,
            check,
            audio,
        }
    }

    pub fn batch(&self) -> &BatchId {
        &self.batch
    }

    pub const fn raw_present(&self) -> bool {
        self.raw_present
    }

    pub const fn source_present(&self) -> bool {
        self.source_present
    }

    pub const fn cards_present(&self) -> bool {
        self.cards_present
    }

    pub const fn check(&self) -> CheckState {
        self.check
    }

    pub const fn audio(&self) -> AudioCoverage {
        self.audio
    }
}
