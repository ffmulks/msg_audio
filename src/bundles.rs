//! Audio bundles for spawning music and sound effects.

use bevy::prelude::*;

use crate::components::{MaxConcurrent, PlaybackRandomizer};
use crate::traits::{MusicCategory, SfxCategory};

/// Default maximum concurrent instances for sound effects.
pub const DEFAULT_MAX_CONCURRENT: u32 = 5;

/// Default timer interval for concurrency checking (in seconds).
pub const DEFAULT_CONCURRENCY_INTERVAL: f32 = 0.5;

/// Bundle for spawning music audio.
///
/// Music uses looping playback by default. The category determines
/// which volume setting applies from the audio configuration.
///
/// # Example
///
/// ```rust,ignore
/// use dmg_audio::MusicBundle;
///
/// commands.spawn((
///     Name::new("Background Music"),
///     MusicBundle::new(music_handle, MyMusicCategory::Exploration),
/// ));
/// ```
#[derive(Bundle)]
pub struct MusicBundle<C: MusicCategory> {
    /// The audio player component.
    pub audio_player: AudioPlayer,
    /// Playback settings (defaults to looping).
    pub playback: PlaybackSettings,
    /// The music category for volume control.
    pub category: C,
}

impl<C: MusicCategory> MusicBundle<C> {
    /// Creates a new music bundle with looping playback.
    #[must_use]
    pub fn new(handle: Handle<AudioSource>, category: C) -> Self {
        Self {
            audio_player: AudioPlayer(handle),
            playback: PlaybackSettings::LOOP,
            category,
        }
    }

    /// Creates a new music bundle with custom playback settings.
    #[must_use]
    pub fn with_settings(
        handle: Handle<AudioSource>,
        category: C,
        playback: PlaybackSettings,
    ) -> Self {
        Self {
            audio_player: AudioPlayer(handle),
            playback,
            category,
        }
    }
}

/// Bundle for spawning sound effect audio.
///
/// Sound effects use despawn-on-finish playback by default.
/// Includes concurrency limiting to prevent audio spam.
///
/// # Example
///
/// ```rust,ignore
/// use dmg_audio::SfxBundle;
///
/// // Basic usage
/// commands.spawn(SfxBundle::new(sound_handle, MySfxCategory::Gameplay));
///
/// // With randomization
/// commands.spawn(SfxBundle::new(sound_handle, MySfxCategory::Gameplay).randomized());
///
/// // With custom settings
/// commands.spawn(
///     SfxBundle::new(sound_handle, MySfxCategory::UI)
///         .with_volume(0.5, 0.8)
///         .with_speed(0.9, 1.1)
///         .with_max_concurrent(3)
/// );
/// ```
#[derive(Bundle)]
pub struct SfxBundle<C: SfxCategory> {
    /// The audio player component.
    pub audio_player: AudioPlayer,
    /// Playback settings (defaults to despawn on finish).
    pub playback: PlaybackSettings,
    /// The sound effect category for volume control.
    pub category: C,
    /// Concurrency limiting component.
    pub max_concurrent: MaxConcurrent,
}

impl<C: SfxCategory> SfxBundle<C> {
    /// Creates a new sound effect bundle with default settings.
    ///
    /// Uses despawn-on-finish playback and default max concurrency (5).
    #[must_use]
    pub fn new(handle: Handle<AudioSource>, category: C) -> Self {
        Self {
            audio_player: AudioPlayer(handle.clone()),
            playback: PlaybackSettings::DESPAWN,
            category,
            max_concurrent: MaxConcurrent::new(handle, DEFAULT_MAX_CONCURRENT),
        }
    }

    /// Sets the volume randomization range.
    ///
    /// The actual volume will be randomly chosen between `min` and `max`.
    #[must_use]
    pub fn with_volume(mut self, min: f32, max: f32) -> Self {
        PlaybackRandomizer::new()
            .with_volume(min, max)
            .apply(&mut self.playback);
        self
    }

    /// Sets the speed (pitch) randomization range.
    ///
    /// The actual speed will be randomly chosen between `min` and `max`.
    #[must_use]
    pub fn with_speed(mut self, min: f32, max: f32) -> Self {
        PlaybackRandomizer::new()
            .with_speed(min, max)
            .apply(&mut self.playback);
        self
    }

    /// Sets the maximum number of concurrent instances of this sound.
    #[must_use]
    pub fn with_max_concurrent(mut self, max: u32) -> Self {
        self.max_concurrent.max = max;
        self
    }

    /// Applies standard randomization (speed 0.7-1.3, volume 0.6-1.0).
    #[must_use]
    pub fn randomized(mut self) -> Self {
        PlaybackRandomizer::standard().apply(&mut self.playback);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
    #[reflect(Component)]
    enum TestMusic {
        #[default]
        Main,
    }

    #[derive(Resource, Clone, Default)]
    struct TestConfig;

    impl crate::traits::AudioCategory for TestMusic {
        type Config = TestConfig;

        fn volume_multiplier(&self, _config: &Self::Config) -> f32 {
            1.0
        }
    }

    impl MusicCategory for TestMusic {}

    #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
    #[reflect(Component)]
    enum TestSfx {
        #[default]
        UI,
    }

    impl crate::traits::AudioCategory for TestSfx {
        type Config = TestConfig;

        fn volume_multiplier(&self, _config: &Self::Config) -> f32 {
            1.0
        }
    }

    impl SfxCategory for TestSfx {}

    #[test]
    fn music_bundle_uses_loop_playback() {
        use bevy::audio::PlaybackMode;

        let handle = Handle::default();
        let bundle = MusicBundle::new(handle, TestMusic::Main);

        assert!(matches!(bundle.playback.mode, PlaybackMode::Loop));
    }

    #[test]
    fn sfx_bundle_uses_despawn_playback() {
        use bevy::audio::PlaybackMode;

        let handle = Handle::default();
        let bundle = SfxBundle::new(handle, TestSfx::UI);

        // DESPAWN mode despawns after playback
        assert!(matches!(bundle.playback.mode, PlaybackMode::Despawn));
    }

    #[test]
    fn sfx_bundle_default_max_concurrent() {
        let handle = Handle::default();
        let bundle = SfxBundle::new(handle, TestSfx::UI);

        assert_eq!(bundle.max_concurrent.max, DEFAULT_MAX_CONCURRENT);
    }

    #[test]
    fn sfx_bundle_with_max_concurrent() {
        let handle = Handle::default();
        let bundle = SfxBundle::new(handle, TestSfx::UI).with_max_concurrent(3);

        assert_eq!(bundle.max_concurrent.max, 3);
    }
}
