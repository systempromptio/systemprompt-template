use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksums: Option<FileChecksums>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_specific: Option<TypeSpecificMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TypeSpecificMetadata {
    Image(ImageMetadata),
    Document(DocumentMetadata),
    Audio(AudioMetadata),
    Video(VideoMetadata),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<ImageGenerationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationInfo {
    pub prompt: String,
    pub model: String,
    pub provider: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_time_ms: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_estimate: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VideoMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChecksums {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

impl FileMetadata {
    pub const fn new() -> Self {
        Self {
            checksums: None,
            type_specific: None,
        }
    }

    pub fn with_image(mut self, image: ImageMetadata) -> Self {
        self.type_specific = Some(TypeSpecificMetadata::Image(image));
        self
    }

    pub fn with_document(mut self, doc: DocumentMetadata) -> Self {
        self.type_specific = Some(TypeSpecificMetadata::Document(doc));
        self
    }

    pub fn with_audio(mut self, audio: AudioMetadata) -> Self {
        self.type_specific = Some(TypeSpecificMetadata::Audio(audio));
        self
    }

    pub fn with_video(mut self, video: VideoMetadata) -> Self {
        self.type_specific = Some(TypeSpecificMetadata::Video(video));
        self
    }

    pub fn with_checksums(mut self, checksums: FileChecksums) -> Self {
        self.checksums = Some(checksums);
        self
    }
}

impl ImageMetadata {
    pub const fn new() -> Self {
        Self {
            width: None,
            height: None,
            alt_text: None,
            description: None,
            generation: None,
        }
    }

    pub const fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_alt_text(mut self, alt: impl Into<String>) -> Self {
        self.alt_text = Some(alt.into());
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_generation(mut self, gen: ImageGenerationInfo) -> Self {
        self.generation = Some(gen);
        self
    }
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageGenerationInfo {
    pub fn new(
        prompt: impl Into<String>,
        model: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            prompt: prompt.into(),
            model: model.into(),
            provider: provider.into(),
            resolution: None,
            aspect_ratio: None,
            generation_time_ms: None,
            cost_estimate: None,
            request_id: None,
        }
    }

    pub fn with_resolution(mut self, resolution: impl Into<String>) -> Self {
        self.resolution = Some(resolution.into());
        self
    }

    pub fn with_aspect_ratio(mut self, aspect_ratio: impl Into<String>) -> Self {
        self.aspect_ratio = Some(aspect_ratio.into());
        self
    }

    pub const fn with_generation_time(mut self, time_ms: i32) -> Self {
        self.generation_time_ms = Some(time_ms);
        self
    }

    pub const fn with_cost_estimate(mut self, cost: f32) -> Self {
        self.cost_estimate = Some(cost);
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl DocumentMetadata {
    pub const fn new() -> Self {
        Self {
            title: None,
            author: None,
            page_count: None,
        }
    }
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioMetadata {
    pub const fn new() -> Self {
        Self {
            duration_seconds: None,
            sample_rate: None,
            channels: None,
        }
    }
}

impl Default for AudioMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoMetadata {
    pub const fn new() -> Self {
        Self {
            width: None,
            height: None,
            duration_seconds: None,
            frame_rate: None,
        }
    }
}

impl Default for VideoMetadata {
    fn default() -> Self {
        Self::new()
    }
}
