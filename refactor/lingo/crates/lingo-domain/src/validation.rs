use crate::{
    CardBatch, Diagnostic, DiagnosticCode, DiagnosticLocation, LanguageProfile,
    RomanisationConvention, SourceBatch, ValidationReport,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn check_card_batch(
    cards: &CardBatch,
    source: &SourceBatch,
    profile: &LanguageProfile,
) -> ValidationReport {
    let mut report = ValidationReport::new();
    check_batch_identity(cards, source, &mut report);
    check_source_coverage(cards, source, &mut report);
    check_card_content(cards, profile, &mut report);
    report
}

fn check_batch_identity(cards: &CardBatch, source: &SourceBatch, report: &mut ValidationReport) {
    if cards.batch_id() != source.batch_id() {
        report.push(Diagnostic::new(
            DiagnosticCode::BatchMismatch,
            DiagnosticLocation::batch(cards.batch_id().clone()),
            format!(
                "card batch {} does not match source batch {}",
                cards.batch_id(),
                source.batch_id()
            ),
        ));
    }
}

fn check_source_coverage(cards: &CardBatch, source: &SourceBatch, report: &mut ValidationReport) {
    let sources = source
        .items()
        .iter()
        .map(|item| (item.id().clone(), item))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();

    for card in cards.cards() {
        let location = DiagnosticLocation::batch(cards.batch_id().clone())
            .with_card(card.id().clone())
            .with_source_item(card.source().item().clone());
        let Some(source_item) = sources.get(card.source().item()) else {
            report.push(Diagnostic::new(
                DiagnosticCode::MissingSourceItem,
                location,
                "card refers to a source item that is not present",
            ));
            continue;
        };
        if !seen.insert(card.source().item().clone()) {
            report.push(Diagnostic::new(
                DiagnosticCode::DuplicateSourceItem,
                location.clone(),
                "more than one card refers to this source item",
            ));
        }
        if card.source().fingerprint() != source_item.fingerprint() {
            report.push(Diagnostic::new(
                DiagnosticCode::FingerprintDrift,
                location.clone().with_field("source.fingerprint"),
                "card lineage fingerprint no longer matches the source item",
            ));
        }
        if card.target() != source_item.target()
            || card.romanisation() != source_item.romanisation()
            || card.english() != source_item.english()
        {
            report.push(Diagnostic::new(
                DiagnosticCode::SourceContentDrift,
                location,
                "card source fields differ from the canonical source item",
            ));
        }
    }

    for source_item in source.items() {
        if !seen.contains(source_item.id()) {
            report.push(Diagnostic::new(
                DiagnosticCode::MissingSourceItem,
                DiagnosticLocation::batch(source.batch_id().clone())
                    .with_source_item(source_item.id().clone()),
                "source item has no card",
            ));
        }
    }
}

fn check_card_content(cards: &CardBatch, profile: &LanguageProfile, report: &mut ValidationReport) {
    for card in cards.cards() {
        let base = DiagnosticLocation::batch(cards.batch_id().clone()).with_card(card.id().clone());
        if profile.romanisation().is_required() && card.romanisation().is_none() {
            report.push(Diagnostic::new(
                DiagnosticCode::MissingRomanisation,
                base.clone().with_field("romanisation"),
                "this language profile requires card romanisation",
            ));
        }
        if let Some(romanisation) = card.romanisation() {
            check_romanisation(
                romanisation.as_str(),
                profile.romanisation(),
                base.clone().with_field("romanisation"),
                report,
            );
            let sentence_tokens = visible_parts(romanisation.as_str());
            let token_parts = card
                .tokens()
                .iter()
                .map(|token| token.romanisation().map(|value| value.as_str()))
                .collect::<Option<Vec<_>>>();
            if let Some(token_parts) = token_parts {
                if sentence_tokens != token_parts {
                    report.push(Diagnostic::new(
                        DiagnosticCode::RomanisationReconstructionMismatch,
                        base.clone().with_field("tokens.romanisation"),
                        "token romanisation does not reconstruct the card romanisation",
                    ));
                }
            }
        }
        let target_parts = visible_parts(card.target().as_str());
        let token_targets = card
            .tokens()
            .iter()
            .map(|token| token.target().as_str())
            .collect::<Vec<_>>();
        if target_parts != token_targets {
            report.push(Diagnostic::new(
                DiagnosticCode::TokenTextMismatch,
                base.clone().with_field("tokens.target"),
                "token target text does not match the visible words in the card target",
            ));
        }
        for (index, token) in card.tokens().iter().enumerate() {
            if profile.romanisation().is_required() && token.romanisation().is_none() {
                report.push(Diagnostic::new(
                    DiagnosticCode::MissingRomanisation,
                    base.clone()
                        .with_field("tokens.romanisation")
                        .with_index(index),
                    "target-language token has no romanisation",
                ));
            }
            if let Some(romanisation) = token.romanisation() {
                check_romanisation(
                    romanisation.as_str(),
                    profile.romanisation(),
                    base.clone()
                        .with_field("tokens.romanisation")
                        .with_index(index),
                    report,
                );
            }
        }
        for (index, word) in card.words().iter().enumerate() {
            if profile.romanisation().is_required() && word.romanisation().is_none() {
                report.push(Diagnostic::new(
                    DiagnosticCode::MissingRomanisation,
                    base.clone()
                        .with_field("words.romanisation")
                        .with_index(index),
                    "word entry has no romanisation",
                ));
            }
        }
        if card.audio().is_none() {
            report.push(Diagnostic::new(
                DiagnosticCode::MissingAudio,
                base.with_field("audio"),
                "card has no audio reference",
            ));
        }
    }
}

fn visible_parts(value: &str) -> Vec<&str> {
    value
        .split(|character: char| character.is_whitespace() || is_punctuation(character))
        .filter(|part| !part.is_empty())
        .collect()
}

fn is_punctuation(character: char) -> bool {
    character.is_ascii_punctuation()
        || matches!(
            character,
            '।' | '॥'
                | '“'
                | '”'
                | '‘'
                | '’'
                | '…'
                | '—'
                | '–'
                | '‐'
                | '‑'
                | '‒'
                | '―'
        )
}

fn check_romanisation(
    value: &str,
    convention: RomanisationConvention,
    location: DiagnosticLocation,
    report: &mut ValidationReport,
) {
    let forbidden: &[&str] = match convention {
        RomanisationConvention::IastTilde => &["ṃ", "ṁ", "m̐"],
        RomanisationConvention::Hepburn | RomanisationConvention::None => &[],
    };
    for fragment in forbidden {
        if value.contains(fragment) {
            report.push(Diagnostic::new(
                DiagnosticCode::ForbiddenRomanisation,
                location.clone(),
                format!("romanisation contains off-profile fragment {fragment:?}"),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::check_card_batch;
    use crate::{
        BatchId, Card, CardBatch, CardId, CardTags, CardToken, Gloss, LanguageCode, LanguageName,
        LanguageProfile, ProfileId, Register, RomanisationConvention, ScriptName, SourceBatch,
        SourceFingerprint, SourceItem, SourceItemId, SourceRef, SourceTags, SourceTitle,
        TargetText, TextDirection, Word, WordId, WordKind,
    };

    #[test]
    fn detects_missing_romanisation() {
        let batch_id = BatchId::parse("chapter-01").unwrap();
        let item_id = SourceItemId::parse("s-1234567890abcdef-01").unwrap();
        let fingerprint = SourceFingerprint::parse(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        let source_item = SourceItem::new(
            item_id.clone(),
            TargetText::parse("यह").unwrap(),
            None,
            Gloss::parse("this").unwrap(),
            SourceTags::default(),
            fingerprint.clone(),
        );
        let source = SourceBatch::try_new(
            batch_id.clone(),
            SourceTitle::parse("Chapter").unwrap(),
            None,
            vec![source_item],
        )
        .unwrap();
        let word = Word::new(
            WordId::parse("w1").unwrap(),
            TargetText::parse("यह").unwrap(),
            None,
            Gloss::parse("this").unwrap(),
            WordKind::Pronoun,
            [],
        );
        let card = Card::try_new(
            CardId::new(batch_id.clone(), item_id.clone()),
            TargetText::parse("यह").unwrap(),
            None,
            Gloss::parse("this").unwrap(),
            Gloss::parse("this").unwrap(),
            Register::Standard,
            vec![CardToken::new(
                TargetText::parse("यह").unwrap(),
                None,
                WordId::parse("w1").unwrap(),
            )],
            vec![word],
            CardTags::default(),
            SourceRef::new(batch_id.clone(), item_id, fingerprint),
        )
        .unwrap();
        let cards = CardBatch::try_new(
            batch_id,
            SourceTitle::parse("Chapter").unwrap(),
            None,
            vec![card],
        )
        .unwrap();
        let profile = LanguageProfile::new(
            ProfileId::parse("hindi").unwrap(),
            LanguageName::parse("Hindi").unwrap(),
            LanguageCode::parse("hi").unwrap(),
            ScriptName::parse("Devanagari").unwrap(),
            TextDirection::Ltr,
            RomanisationConvention::IastTilde,
        );

        let report = check_card_batch(&cards, &source, &profile);
        assert!(!report.is_clean());
    }

    #[test]
    fn detects_token_romanisation_mismatch() {
        let batch_id = BatchId::parse("chapter-01").unwrap();
        let item_id = SourceItemId::parse("s-1234567890abcdef-01").unwrap();
        let fingerprint = SourceFingerprint::parse(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap();
        let source_item = SourceItem::new(
            item_id.clone(),
            TargetText::parse("यह किताब").unwrap(),
            Some(crate::Romanisation::parse("yah kitāb").unwrap()),
            Gloss::parse("this book").unwrap(),
            SourceTags::default(),
            fingerprint.clone(),
        );
        let source = SourceBatch::try_new(
            batch_id.clone(),
            SourceTitle::parse("Chapter").unwrap(),
            None,
            vec![source_item],
        )
        .unwrap();
        let words = vec![
            Word::new(
                WordId::parse("w1").unwrap(),
                TargetText::parse("यह").unwrap(),
                Some(crate::Romanisation::parse("yah").unwrap()),
                Gloss::parse("this").unwrap(),
                WordKind::Pronoun,
                [],
            ),
            Word::new(
                WordId::parse("w2").unwrap(),
                TargetText::parse("किताब").unwrap(),
                Some(crate::Romanisation::parse("galat").unwrap()),
                Gloss::parse("book").unwrap(),
                WordKind::Noun,
                [],
            ),
        ];
        let tokens = vec![
            CardToken::new(
                TargetText::parse("यह").unwrap(),
                Some(crate::Romanisation::parse("yah").unwrap()),
                WordId::parse("w1").unwrap(),
            ),
            CardToken::new(
                TargetText::parse("किताब").unwrap(),
                Some(crate::Romanisation::parse("galat").unwrap()),
                WordId::parse("w2").unwrap(),
            ),
        ];
        let card = Card::try_new(
            CardId::new(batch_id.clone(), item_id.clone()),
            TargetText::parse("यह किताब").unwrap(),
            Some(crate::Romanisation::parse("yah kitāb").unwrap()),
            Gloss::parse("this book").unwrap(),
            Gloss::parse("this book").unwrap(),
            Register::Standard,
            tokens,
            words,
            CardTags::default(),
            SourceRef::new(batch_id.clone(), item_id, fingerprint),
        )
        .unwrap();
        let cards = CardBatch::try_new(
            batch_id,
            SourceTitle::parse("Chapter").unwrap(),
            None,
            vec![card],
        )
        .unwrap();
        let profile = LanguageProfile::new(
            ProfileId::parse("hindi").unwrap(),
            LanguageName::parse("Hindi").unwrap(),
            LanguageCode::parse("hi").unwrap(),
            ScriptName::parse("Devanagari").unwrap(),
            TextDirection::Ltr,
            RomanisationConvention::IastTilde,
        );

        let report = check_card_batch(&cards, &source, &profile);
        assert!(report.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == crate::DiagnosticCode::RomanisationReconstructionMismatch
        }));
    }
}
