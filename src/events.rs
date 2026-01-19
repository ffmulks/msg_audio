//! Message-based audio playback API.
//!
//! This module provides messages for triggering audio playback without
//! directly spawning entities. Useful for fire-and-forget sounds.
//!
//! ## Music Messages
//!
//! - [`PlayMusic`] - Start playing a music track
//! - [`StopMusic`] - Stop a specific music category
//! - [`StopAllMusic`] - Stop all currently playing music
//! - [`FadeOutMusic`] - Gradually fade out music over time
//!
//! ## Sound Effect Messages
//!
//! - [`PlaySfx`] - Play a sound effect

use bevy::prelude::*;
use std::time::Duration;

use crate::components::{Audio, PlaybackRandomizer};
use crate::traits::{MusicCategory, SfxCategory};

/// Extracts a display name from an audio handle path.
///
/// Returns the filename without extension, or "Audio" if no path is available.
fn audio_name_from_handle(asset_server: &AssetServer, handle: &Handle<AudioSource>) -> String {
    asset_server
        .get_path(handle)
        .and_then(|path| {
            path.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "Audio".to_string())
}

/// Message to request playing a music track.
///
/// When triggered, spawns a music entity with the specified settings.
///
/// # Example
///
/// ```rust,ignore
/// use msg_audio::PlayMusic;
///
/// fn start_level_music(mut messages: MessageWriter<PlayMusic<MyMusicCategory>>) {
///     messages.write(PlayMusic::new(music_handle, MyMusicCategory::Exploration));
/// }
/// ```
#[derive(Message, Clone)]
pub struct PlayMusic<M: MusicCategory> {
    /// Handle to the audio source.
    pub handle: Handle<AudioSource>,
    /// The music category for volume control.
    pub category: M,
    /// Custom playback settings (defaults to LOOP).
    pub playback: PlaybackSettings,
}

impl<M: MusicCategory> PlayMusic<M> {
    /// Creates a new play music event with looping playback.
    #[must_use]
    pub fn new(handle: Handle<AudioSource>, category: M) -> Self {
        Self {
            handle,
            category,
            playback: PlaybackSettings::LOOP,
        }
    }

    /// Sets custom playback settings.
    #[must_use]
    pub fn with_playback(mut self, playback: PlaybackSettings) -> Self {
        self.playback = playback;
        self
    }
}

/// Message to stop music of a specific category.
///
/// When triggered, immediately stops and despawns all music entities
/// matching the specified category.
///
/// # Example
///
/// ```rust,ignore
/// use msg_audio::StopMusic;
///
/// fn stop_combat_music(mut messages: MessageWriter<StopMusic<MyMusicCategory>>) {
///     messages.write(StopMusic::new(MyMusicCategory::Combat));
/// }
/// ```
#[derive(Message, Clone)]
pub struct StopMusic<M: MusicCategory> {
    /// The music category to stop.
    pub category: M,
}

impl<M: MusicCategory> StopMusic<M> {
    /// Creates a new stop music event.
    #[must_use]
    pub fn new(category: M) -> Self {
        Self { category }
    }
}

/// Message to stop all currently playing music.
///
/// When triggered, immediately stops and despawns all music entities
/// regardless of category.
///
/// # Example
///
/// ```rust,ignore
/// use msg_audio::StopAllMusic;
///
/// fn mute_all_music(mut messages: MessageWriter<StopAllMusic<MyMusicCategory>>) {
///     messages.write(StopAllMusic::default());
/// }
/// ```
#[derive(Message, Clone, Default)]
pub struct StopAllMusic<M: MusicCategory> {
    _phantom: std::marker::PhantomData<M>,
}

/// Message to fade out music of a specific category.
///
/// Gradually reduces the volume of matching music entities over the
/// specified duration, then despawns them.
///
/// # Example
///
/// ```rust,ignore
/// use msg_audio::FadeOutMusic;
/// use std::time::Duration;
///
/// fn fade_to_new_track(mut messages: MessageWriter<FadeOutMusic<MyMusicCategory>>) {
///     messages.write(FadeOutMusic::new(
///         MyMusicCategory::Exploration,
///         Duration::from_secs(2),
///     ));
/// }
/// ```
#[derive(Message, Clone)]
pub struct FadeOutMusic<M: MusicCategory> {
    /// The music category to fade out.
    pub category: M,
    /// Duration of the fade-out effect.
    pub duration: Duration,
}

impl<M: MusicCategory> FadeOutMusic<M> {
    /// Creates a new fade-out music event.
    #[must_use]
    pub fn new(category: M, duration: Duration) -> Self {
        Self { category, duration }
    }

    /// Creates a fade-out event with a duration in seconds.
    #[must_use]
    pub fn from_secs(category: M, seconds: f32) -> Self {
        Self {
            category,
            duration: Duration::from_secs_f32(seconds),
        }
    }
}

/// Message to request playing a sound effect.
///
/// When triggered, spawns a sound effect entity with the specified settings.
///
/// # Example
///
/// ```rust,ignore
/// use msg_audio::PlaySfx;
///
/// fn play_hit_sound(mut messages: MessageWriter<PlaySfx<MySfxCategory>>) {
///     messages.write(
///         PlaySfx::new(hit_sound_handle, MySfxCategory::Gameplay)
///             .randomized()
///             .with_max_concurrent(3)
///     );
/// }
/// ```
#[derive(Message, Clone)]
pub struct PlaySfx<S: SfxCategory> {
    /// Handle to the audio source.
    pub handle: Handle<AudioSource>,
    /// The sound effect category for volume control.
    pub category: S,
    /// Custom playback settings (defaults to DESPAWN).
    pub playback: PlaybackSettings,
    /// Maximum concurrent instances of this sound.
    pub max_concurrent: u32,
}

impl<S: SfxCategory> PlaySfx<S> {
    /// Creates a new play sound effect event.
    #[must_use]
    pub fn new(handle: Handle<AudioSource>, category: S) -> Self {
        Self {
            handle,
            category,
            playback: PlaybackSettings::DESPAWN,
            max_concurrent: crate::bundles::DEFAULT_MAX_CONCURRENT,
        }
    }

    /// Sets custom playback settings.
    #[must_use]
    pub fn with_playback(mut self, playback: PlaybackSettings) -> Self {
        self.playback = playback;
        self
    }

    /// Sets the maximum concurrent instances.
    #[must_use]
    pub fn with_max_concurrent(mut self, max: u32) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Sets volume randomization range.
    #[must_use]
    pub fn with_volume(mut self, min: f32, max: f32) -> Self {
        PlaybackRandomizer::new()
            .with_volume(min, max)
            .apply(&mut self.playback);
        self
    }

    /// Sets speed randomization range.
    #[must_use]
    pub fn with_speed(mut self, min: f32, max: f32) -> Self {
        PlaybackRandomizer::new()
            .with_speed(min, max)
            .apply(&mut self.playback);
        self
    }

    /// Applies standard randomization (speed 0.7-1.3, volume 0.6-1.0).
    #[must_use]
    pub fn randomized(mut self) -> Self {
        PlaybackRandomizer::standard().apply(&mut self.playback);
        self
    }
}

/// System that handles `PlayMusic` messages by spawning music entities.
///
/// If an [`Audio`] entity exists, spawned music will be parented to it.
/// Each spawned entity receives a `Name` component based on the audio file path.
pub fn handle_play_music_events<M: MusicCategory>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut messages: MessageReader<PlayMusic<M>>,
    audio_parent: Query<Entity, With<Audio>>,
) {
    let parent = audio_parent.iter().next();

    for event in messages.read() {
        let name = audio_name_from_handle(&asset_server, &event.handle);
        let mut entity_commands = commands.spawn((
            Name::new(name),
            AudioPlayer(event.handle.clone()),
            event.playback,
            event.category,
        ));

        if let Some(parent_entity) = parent {
            entity_commands.set_parent_in_place(parent_entity);
        }
    }
}

/// System that handles `PlaySfx` messages by spawning sound effect entities.
///
/// If an [`Audio`] entity exists, spawned sound effects will be parented to it.
/// Each spawned entity receives a `Name` component based on the audio file path.
pub fn handle_play_sfx_events<S: SfxCategory>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut messages: MessageReader<PlaySfx<S>>,
    audio_parent: Query<Entity, With<Audio>>,
) {
    use crate::components::MaxConcurrent;

    let parent = audio_parent.iter().next();

    for event in messages.read() {
        let name = audio_name_from_handle(&asset_server, &event.handle);
        let mut entity_commands = commands.spawn((
            Name::new(name),
            AudioPlayer(event.handle.clone()),
            event.playback,
            event.category,
            MaxConcurrent::new(event.handle.clone(), event.max_concurrent),
        ));

        if let Some(parent_entity) = parent {
            entity_commands.set_parent_in_place(parent_entity);
        }
    }
}

/// System that handles `StopMusic` messages by despawning matching music entities.
pub fn handle_stop_music_events<M: MusicCategory>(
    mut commands: Commands,
    mut messages: MessageReader<StopMusic<M>>,
    query: Query<(Entity, &M)>,
) {
    for event in messages.read() {
        for (entity, category) in &query {
            if *category == event.category {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// System that handles `StopAllMusic` messages by despawning all music entities.
pub fn handle_stop_all_music_events<M: MusicCategory>(
    mut commands: Commands,
    mut messages: MessageReader<StopAllMusic<M>>,
    query: Query<Entity, With<M>>,
) {
    for _ in messages.read() {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}

/// System that handles `FadeOutMusic` messages by adding fade-out components.
pub fn handle_fade_out_music_events<M: MusicCategory>(
    mut commands: Commands,
    mut messages: MessageReader<FadeOutMusic<M>>,
    query: Query<(Entity, &M, &AudioSink)>,
) {
    use crate::components::FadeOut;
    use bevy::audio::Volume;

    for event in messages.read() {
        for (entity, category, sink) in &query {
            if *category == event.category {
                // Get current volume to use as initial fade volume
                let initial_volume = match sink.volume() {
                    Volume::Linear(v) => v,
                    Volume::Decibels(db) => 10_f32.powf(db / 20.0),
                };
                commands
                    .entity(entity)
                    .insert(FadeOut::new(event.duration).with_initial_volume(initial_volume));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
    #[reflect(Component)]
    enum TestSfx {
        #[default]
        UI,
    }

    #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
    #[reflect(Component)]
    enum TestMusic {
        #[default]
        MainMenu,
        Gameplay,
    }

    #[derive(Resource, Clone, Default)]
    struct TestConfig;

    impl crate::traits::AudioCategory for TestSfx {
        type Config = TestConfig;
        fn volume_multiplier(&self, _: &Self::Config) -> f32 {
            1.0
        }
    }

    impl SfxCategory for TestSfx {}

    impl crate::traits::AudioCategory for TestMusic {
        type Config = TestConfig;
        fn volume_multiplier(&self, _: &Self::Config) -> f32 {
            1.0
        }
    }

    impl MusicCategory for TestMusic {}

    #[test]
    fn play_sfx_default_max_concurrent() {
        let event = PlaySfx::new(Handle::default(), TestSfx::UI);
        assert_eq!(event.max_concurrent, crate::bundles::DEFAULT_MAX_CONCURRENT);
    }

    #[test]
    fn play_sfx_with_max_concurrent() {
        let event = PlaySfx::new(Handle::default(), TestSfx::UI).with_max_concurrent(3);
        assert_eq!(event.max_concurrent, 3);
    }

    #[test]
    fn play_music_defaults_to_loop() {
        use bevy::audio::PlaybackMode;

        let event = PlayMusic::new(Handle::default(), TestMusic::MainMenu);
        assert!(matches!(event.playback.mode, PlaybackMode::Loop));
    }

    #[test]
    fn play_music_with_playback_settings() {
        let event = PlayMusic::new(Handle::default(), TestMusic::Gameplay)
            .with_playback(PlaybackSettings::ONCE);

        assert!(matches!(
            event.playback.mode,
            bevy::audio::PlaybackMode::Once
        ));
    }

    #[test]
    fn stop_music_new() {
        let event = StopMusic::new(TestMusic::Gameplay);
        assert_eq!(event.category, TestMusic::Gameplay);
    }

    #[test]
    fn stop_all_music_default() {
        let _event: StopAllMusic<TestMusic> = StopAllMusic::default();
        // Just verify it can be created
    }

    #[test]
    fn fade_out_music_new() {
        let event = FadeOutMusic::new(TestMusic::MainMenu, Duration::from_secs(2));

        assert_eq!(event.category, TestMusic::MainMenu);
        assert_eq!(event.duration, Duration::from_secs(2));
    }

    #[test]
    fn fade_out_music_from_secs() {
        let event = FadeOutMusic::from_secs(TestMusic::Gameplay, 1.5);

        assert_eq!(event.category, TestMusic::Gameplay);
        assert!((event.duration.as_secs_f32() - 1.5).abs() < 0.001);
    }
}
