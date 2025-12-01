//! Core traits for audio category types.
//!
//! Games implement these traits for their own audio category enums to enable
//! pluggable volume control per category.

use bevy::prelude::*;

/// Base trait for audio categories that provide volume multipliers.
///
/// This trait is implemented by both music and sound effect category types.
/// Each category can have its own volume level in the audio configuration.
pub trait AudioCategory: Component + Clone + Copy + Default + PartialEq + Send + Sync + 'static {
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
///     pub muted: bool,
/// }
///
/// impl AudioConfigTrait for MyAudioConfig {
///     fn master_volume(&self) -> f32 {
///         self.master
///     }
///
///     fn is_muted(&self) -> bool {
///         self.muted
///     }
/// }
/// ```
pub trait AudioConfigTrait: Resource + Clone + Default + Send + Sync + 'static {
    /// Returns the master volume level.
    ///
    /// This value is multiplied with category volumes to get the final volume.
    /// Range: [0.0, 1.0]
    fn master_volume(&self) -> f32;

    /// Returns whether audio is globally muted.
    ///
    /// When muted, all audio should play at zero volume.
    /// Default implementation returns `false`.
    fn is_muted(&self) -> bool {
        false
    }

    /// Returns the effective master volume, accounting for mute state.
    ///
    /// Returns 0.0 if muted, otherwise returns [`master_volume()`](Self::master_volume).
    fn effective_volume(&self) -> f32 {
        if self.is_muted() {
            0.0
        } else {
            self.master_volume()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Clone, Default)]
    struct TestConfigWithMute {
        master: f32,
        muted: bool,
    }

    impl AudioConfigTrait for TestConfigWithMute {
        fn master_volume(&self) -> f32 {
            self.master
        }

        fn is_muted(&self) -> bool {
            self.muted
        }
    }

    #[derive(Resource, Clone, Default)]
    struct TestConfigWithoutMute {
        master: f32,
    }

    impl AudioConfigTrait for TestConfigWithoutMute {
        fn master_volume(&self) -> f32 {
            self.master
        }
        // Uses default is_muted() which returns false
    }

    #[test]
    fn effective_volume_when_not_muted() {
        let config = TestConfigWithMute {
            master: 0.8,
            muted: false,
        };

        assert!((config.effective_volume() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn effective_volume_when_muted() {
        let config = TestConfigWithMute {
            master: 0.8,
            muted: true,
        };

        assert!((config.effective_volume() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_is_muted_returns_false() {
        let config = TestConfigWithoutMute { master: 0.5 };

        assert!(!config.is_muted());
        assert!((config.effective_volume() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn master_volume_is_independent_of_mute() {
        let config = TestConfigWithMute {
            master: 0.75,
            muted: true,
        };

        // master_volume() returns the raw value regardless of mute state
        assert!((config.master_volume() - 0.75).abs() < f32::EPSILON);
        // effective_volume() accounts for mute
        assert!((config.effective_volume() - 0.0).abs() < f32::EPSILON);
    }
}
