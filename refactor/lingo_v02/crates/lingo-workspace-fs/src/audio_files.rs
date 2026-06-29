use lingo_application::ports::{AudioFileFailure, AudioFileStore, SynthesizedAudio};
use lingo_domain::{AudioRelativePath, ContentHash, SentenceAudio, SentenceId};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct FsAudioFileStore {
    root: PathBuf,
}

impl FsAudioFileStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    fn audio_dir(&self) -> &Path { &self.root }
}

impl AudioFileStore for FsAudioFileStore {
    fn write_sentence_audio(&self, sentence: &SentenceId, audio: &SynthesizedAudio) -> Result<SentenceAudio, AudioFileFailure> {
        std::fs::create_dir_all(self.audio_dir()).map_err(|error| AudioFileFailure::Io(error.to_string()))?;
        let file_name = format!("{}.{}", sentence.as_str(), audio.format.extension());
        let tmp = self.audio_dir().join(format!(".{file_name}.tmp"));
        let dest = self.audio_dir().join(&file_name);
        std::fs::write(&tmp, &audio.bytes).map_err(|error| AudioFileFailure::Io(error.to_string()))?;
        let read_back = std::fs::read(&tmp).map_err(|error| AudioFileFailure::Io(error.to_string()))?;
        if read_back != audio.bytes { return Err(AudioFileFailure::Verification); }
        std::fs::rename(&tmp, &dest).map_err(|error| AudioFileFailure::Io(error.to_string()))?;
        let relative = AudioRelativePath::parse(format!("audio/{file_name}"))
            .map_err(|error| AudioFileFailure::Io(error.to_string()))?;
        Ok(SentenceAudio::new(
            relative,
            ContentHash::sha256(&audio.bytes),
            audio.backend,
            audio.format,
            audio.voice.clone(),
            audio.model.clone(),
        ))
    }
}
