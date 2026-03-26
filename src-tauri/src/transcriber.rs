use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use whisper_rs::{WhisperContext, WhisperContextParameters};

pub trait ModelDirectory {
    fn transcriber_model_dir(&self) -> PathBuf;
}

impl ModelDirectory for directories::ProjectDirs {
    fn transcriber_model_dir(&self) -> PathBuf {
        let dir = self.cache_dir().join("model");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}

#[derive(Debug, EnumIter, Serialize)]
pub enum Category {
    Recommended,
    Other,
}

impl Category {
    pub fn name(&self) -> &'static str {
        match self {
            Category::Recommended => "Recommended Models",
            Category::Other => "Other Models",
        }
    }
}

pub enum ModelType {
    Whisper,
    Quantized,
}

impl ModelType {
    pub fn name(&self) -> &'static str {
        match self {
            ModelType::Whisper => "Whisper",
            ModelType::Quantized => "Quantized",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, EnumIter)]
pub enum Model {
    TinyWhisper,
    TinyEnWhisper,
    TinyQuantized,
    TinyEnQuantized,
    BaseWhisper,
    BaseEnWhisper,
    BaseQuantized,
    BaseEnQuantized,
}

impl Model {
    pub fn default_model() -> Self {
        Self::BaseWhisper
    }

    pub fn recommended_models() -> Vec<Self> {
        vec![Self::TinyWhisper, Self::BaseWhisper]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::TinyWhisper => "Tiny",
            Self::TinyEnWhisper => "Tiny English",
            Self::TinyQuantized => "Tiny (Quantized)",
            Self::TinyEnQuantized => "Tiny English (Quantized)",
            Self::BaseWhisper => "Base",
            Self::BaseEnWhisper => "Base English",
            Self::BaseQuantized => "Base (Quantized)",
            Self::BaseEnQuantized => "Base English (Quantized)",
        }
    }

    pub fn download_url(&self) -> &'static str {
        match self {
            Self::TinyWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin?download=true",
            Self::TinyEnWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin?download=true",
            Self::TinyQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny-q5_1.bin?download=true",
            Self::TinyEnQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en-q5_1.bin?download=true",
            Self::BaseWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin?download=true",
            Self::BaseEnWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true",
            Self::BaseQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin?download=true",
            Self::BaseEnQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en-q5_1.bin?download=true",
        }
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            Self::TinyWhisper => "tiny.bin",
            Self::TinyEnWhisper => "tiny-en.bin",
            Self::TinyQuantized => "tiny-q.bin",
            Self::TinyEnQuantized => "tiny-en-q.bin",
            Self::BaseWhisper => "base.bin",
            Self::BaseEnWhisper => "base-en.bin",
            Self::BaseQuantized => "base-q.bin",
            Self::BaseEnQuantized => "base-en-q.bin",
        }
    }

    pub fn average_memory_usage(&self) -> usize {
        match self {
            Self::TinyWhisper | Self::TinyEnWhisper | Self::TinyQuantized | Self::TinyEnQuantized => 390,
            Self::BaseWhisper | Self::BaseEnWhisper | Self::BaseQuantized | Self::BaseEnQuantized => 500,
        }
    }

    pub fn can_run(&self) -> bool {
        use sysinfo::{MemoryRefreshKind, RefreshKind, System};
        let system = System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::everything()));
        let available = (system.total_memory() + system.total_swap()) as usize;
        (self.average_memory_usage() * 1_000_000) < available
    }

    pub fn disk_usage(&self) -> usize {
        match self {
            Self::TinyWhisper | Self::TinyEnWhisper => 77,
            Self::TinyQuantized | Self::TinyEnQuantized => 33,
            Self::BaseWhisper | Self::BaseEnWhisper => 148,
            Self::BaseQuantized | Self::BaseEnQuantized => 60,
        }
    }

    pub fn path(&self) -> PathBuf {
        crate::project_directory().transcriber_model_dir().join(self.file_name())
    }

    pub fn is_downloaded(&self) -> bool {
        self.path().exists()
    }

    pub fn category(&self) -> Category {
        match self {
            Self::TinyWhisper | Self::BaseWhisper => Category::Recommended,
            _ => Category::Other,
        }
    }

    pub fn model_type(&self) -> ModelType {
        match self {
            Self::TinyQuantized | Self::TinyEnQuantized
            | Self::BaseQuantized | Self::BaseEnQuantized => ModelType::Quantized,
            _ => ModelType::Whisper,
        }
    }
}

/// Load a WhisperContext from the downloaded model file.
pub fn load_context(model: &Model) -> anyhow::Result<WhisperContext> {
    let path = model.path();
    if !path.exists() {
        anyhow::bail!("Model {:?} is not downloaded at {:?}", model, path);
    }
    let ctx = WhisperContext::new_with_params(
        path.to_str().unwrap(),
        WhisperContextParameters::default(),
    )?;
    Ok(ctx)
}
