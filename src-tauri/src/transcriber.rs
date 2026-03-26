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

pub enum Type {
    Whisper,
    Quantized,
}

impl Type {
    pub fn name(&self) -> &'static str {
        match self {
            Type::Whisper => "Whisper",
            Type::Quantized => "Quantized",
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
    SmallWhisper,
    SmallEnWhisper,
    SmallQuantized,
    SmallEnQuantized,
    SmallDiarize,
    MediumWhisper,
    MediumQuantized,
    MediumEnQuantized,
    LargeWhisper,
    LargeQuantized,
}

impl Model {
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
            Self::SmallWhisper => "Small",
            Self::SmallEnWhisper => "Small English",
            Self::SmallQuantized => "Small (Quantized)",
            Self::SmallEnQuantized => "Small English (Quantized)",
            Self::SmallDiarize => "Small Diarize",
            Self::MediumWhisper => "Medium",
            Self::MediumQuantized => "Medium (Quantized)",
            Self::MediumEnQuantized => "Medium English (Quantized)",
            Self::LargeWhisper => "Large",
            Self::LargeQuantized => "Large (Quantized)",
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
            Self::SmallWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin?download=true",
            Self::SmallEnWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin?download=true",
            Self::SmallQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small-q5_1.bin?download=true",
            Self::SmallEnQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en-q5_1.bin?download=true",
            Self::SmallDiarize => "https://huggingface.co/akashmjn/tinydiarize-whisper.cpp/resolve/main/ggml-small.en-tdrz.bin?download=true",
            Self::MediumWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin?download=true",
            Self::MediumQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium-q5_0.bin?download=true",
            Self::MediumEnQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en-q5_0.bin?download=true",
            Self::LargeWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin?download=true",
            Self::LargeQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin?download=true",
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
            Self::SmallWhisper => "small.bin",
            Self::SmallEnWhisper => "small-en.bin",
            Self::SmallQuantized => "small-q.bin",
            Self::SmallEnQuantized => "small-en-q.bin",
            Self::SmallDiarize => "small-diar.bin",
            Self::MediumWhisper => "medium.bin",
            Self::MediumQuantized => "medium-q.bin",
            Self::MediumEnQuantized => "medium-en.bin",
            Self::LargeWhisper => "large.bin",
            Self::LargeQuantized => "large-q.bin",
        }
    }

    pub fn average_memory_usage(&self) -> usize {
        match self {
            Self::TinyWhisper | Self::TinyEnWhisper | Self::TinyQuantized | Self::TinyEnQuantized => 390,
            Self::BaseWhisper | Self::BaseEnWhisper | Self::BaseQuantized | Self::BaseEnQuantized => 500,
            Self::SmallWhisper | Self::SmallEnWhisper | Self::SmallQuantized | Self::SmallEnQuantized | Self::SmallDiarize => 1000,
            Self::MediumWhisper | Self::MediumQuantized | Self::MediumEnQuantized => 2600,
            Self::LargeWhisper | Self::LargeQuantized => 4700,
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
            Self::SmallWhisper | Self::SmallEnWhisper | Self::SmallDiarize => 488,
            Self::SmallQuantized | Self::SmallEnQuantized => 190,
            Self::MediumWhisper => 1530,
            Self::MediumQuantized | Self::MediumEnQuantized => 539,
            Self::LargeWhisper => 3100,
            Self::LargeQuantized => 1080,
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
            Self::TinyWhisper | Self::BaseWhisper | Self::SmallWhisper
            | Self::MediumWhisper | Self::MediumQuantized
            | Self::LargeWhisper | Self::LargeQuantized => Category::Recommended,
            _ => Category::Other,
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            Self::TinyQuantized | Self::TinyEnQuantized
            | Self::BaseQuantized | Self::BaseEnQuantized
            | Self::SmallQuantized | Self::SmallEnQuantized
            | Self::SmallDiarize | Self::MediumQuantized
            | Self::MediumEnQuantized | Self::LargeQuantized => Type::Quantized,
            _ => Type::Whisper,
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
