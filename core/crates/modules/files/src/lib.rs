#![allow(clippy::pedantic)]
#![allow(clippy::too_many_arguments)]

pub mod models;
pub mod repository;

pub use models::{
    AudioMetadata, ContentFile, DocumentMetadata, File, FileChecksums, FileMetadata, FileRole,
    ImageGenerationInfo, ImageMetadata, TypeSpecificMetadata, VideoMetadata,
};
pub use repository::FileRepository;

pub type GenerationInfo = ImageGenerationInfo;
