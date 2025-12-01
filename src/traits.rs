//! Core traits for audio category types.
//!
//! Games implement these traits for their own audio category enums to enable
//! pluggable volume control per category.

use bevy::prelude::*;

/// Base trait for audio categories that provide volume multipliers.
///
/// This trait is implemented by both music and sound effect category types.
/// Each category can have its own volume level in the audio configuration.
pub trait AudioCategory: Component + Clone + Copy + Default + Send + Sync + 'static {
    /// The configuration type that provides volume settings for this category.
    type Config: Resource;

    /// Returns the volume multiplier for this category from the configuration.
    ///
    /// The returned value should be in the range [0.0, 1.0].
    fn volume_multiplier(&self, config: &Self::Config) -> f32;
}

/// Marker trait for music categories.
///
/// Music categories typically represent background music types like main menu music,
/// gameplay music, combat music, etc. Music usually loops continuously.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use dmg_audio::{MusicCategory, AudioCategory};
///
/// #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
/// #[reflect(Component)]
/// pub enum GameMusic {
///     MainMenu,
///     #[default]
///     Exploration,
///     Combat,
/// }
///
/// impl AudioCategory for GameMusic {
///     type Config = MyAudioConfig;
///
///     fn volume_multiplier(&self, config: &Self::Config) -> f32 {
///         match self {
///             GameMusic::MainMenu => config.main_menu_music,
///             GameMusic::Exploration => config.exploration_music,
///             GameMusic::Combat => config.combat_music,
///         }
///     }
/// }
///
/// impl MusicCategory for GameMusic {}
/// ```
pub trait MusicCategory: AudioCategory {}

/// Marker trait for sound effect categories.
///
/// Sound effect categories typically represent different types of one-shot sounds
/// like UI clicks, gameplay sounds, ambient sounds, etc.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use dmg_audio::{SfxCategory, AudioCategory};
///
/// #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
/// #[reflect(Component)]
/// pub enum GameSfx {
///     #[default]
///     UI,
///     Gameplay,
///     Ambience,
/// }
///
/// impl AudioCategory for GameSfx {
///     type Config = MyAudioConfig;
///
///     fn volume_multiplier(&self, config: &Self::Config) -> f32 {
///         match self {
///             GameSfx::UI => config.ui_sfx,
///             GameSfx::Gameplay => config.gameplay_sfx,
///             GameSfx::Ambience => config.ambience_sfx,
///         }
///     }
/// }
///
/// impl SfxCategory for GameSfx {}
/// ```
pub trait SfxCategory: AudioCategory {}

/// Trait for audio configuration resources.
///
/// Provides master volume and category volume multipliers.
/// Games implement this trait to define their volume structure.
///
/// # Example
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use dmg_audio::AudioConfigTrait;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
/// #[reflect(Resource)]
/// pub struct MyAudioConfig {
///     pub master: f32,
///     pub music: f32,
///     pub sfx: f32,
/// }
///
/// impl AudioConfigTrait for MyAudioConfig {
///     fn master_volume(&self) -> f32 {
///         self.master
///     }
/// }
/// ```
pub trait AudioConfigTrait: Resource + Clone + Default + Send + Sync + 'static {
    /// Returns the master volume level.
    ///
    /// This value is multiplied with category volumes to get the final volume.
    /// Range: [0.0, 1.0]
    fn master_volume(&self) -> f32;
}
