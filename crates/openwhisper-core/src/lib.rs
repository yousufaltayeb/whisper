pub mod audio;
pub mod capabilities;
pub mod clipboard;
pub mod config;
pub mod models;
pub mod paths;
pub mod processing;
pub mod providers;
pub mod sound;
pub mod state;
pub mod storage;
pub mod streaming;

pub use capabilities::detect_capabilities;
pub use config::{
    AppConfig, AudioBackend, AudioConfig, DeliveryConfig, HistoryConfig, InferenceBackend,
    ModelConfig, OverlayMode, PrivacyConfig,
};
pub use paths::{AppPaths, LegacyDataReport};
pub use sound::{SoundCue, play_sound_cue};
pub use state::{CaptureCommand, CaptureCoordinator, CaptureState, DeliveryTarget, Mode};
pub use storage::{HistoryEntry, HistoryInput, InstalledModel, StateStore};
pub use streaming::{
    AudioCoalescer, StabilizedUpdate, StreamingTextProcessor, TranscriptStabilizer,
};
