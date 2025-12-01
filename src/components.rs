//! Audio components for tracking concurrency and playback settings.

use bevy::{audio::Volume, platform::collections::HashMap, prelude::*};
use rand::prelude::*;

/// Component that limits the maximum concurrent instances of a sound.
///
/// When more than `max` sounds with the same `handle` are playing,
/// the excess sounds are despawned (keeping the first N spawned).
///
/// # Example
///
/// ```rust,ignore
/// use dmg_audio::MaxConcurrent;
///
/// // Limit to 3 concurrent footstep sounds
/// commands.spawn((
///     AudioPlayer(footstep_handle.clone()),
///     MaxConcurrent { handle: footstep_handle, max: 3 },
/// ));
/// ```
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct MaxConcurrent {
    /// The audio source handle to track concurrency for.
    pub handle: Handle<AudioSource>,
    /// Maximum number of concurrent instances allowed.
    pub max: u32,
}

impl MaxConcurrent {
    /// Creates a new `MaxConcurrent` component.
    #[must_use]
    pub fn new(handle: Handle<AudioSource>, max: u32) -> Self {
        Self { handle, max }
    }
}

/// Resource that tracks the count of active sound effects per handle.
///
/// This is used internally by the concurrency limiting system.
#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct SoundEffectCounter {
    /// Map of audio handle to current count of playing instances.
    pub counts: HashMap<Handle<AudioSource>, u32>,
    /// Timer for periodic count resets to prevent stale data.
    pub timer: Timer,
}

impl SoundEffectCounter {
    /// Creates a new counter with the specified reset interval.
    #[must_use]
    pub fn with_interval(seconds: f32) -> Self {
        Self {
            counts: HashMap::default(),
            timer: Timer::from_seconds(seconds, TimerMode::Repeating),
        }
    }
}

/// Builder for randomized playback settings.
///
/// Provides a fluent API for configuring volume and speed randomization
/// on sound effects to add variety.
#[derive(Clone, Debug)]
pub struct PlaybackRandomizer {
    /// Minimum and maximum volume range.
    pub volume_range: Option<(f32, f32)>,
    /// Minimum and maximum speed range.
    pub speed_range: Option<(f32, f32)>,
}

impl Default for PlaybackRandomizer {
    fn default() -> Self {
        Self {
            volume_range: None,
            speed_range: None,
        }
    }
}

impl PlaybackRandomizer {
    /// Creates a new randomizer with no randomization.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the volume randomization range.
    ///
    /// Volume will be randomly chosen between `min` and `max` (inclusive).
    #[must_use]
    pub fn with_volume(mut self, min: f32, max: f32) -> Self {
        self.volume_range = Some((min, max));
        self
    }

    /// Sets the speed (pitch) randomization range.
    ///
    /// Speed will be randomly chosen between `min` and `max` (inclusive).
    #[must_use]
    pub fn with_speed(mut self, min: f32, max: f32) -> Self {
        self.speed_range = Some((min, max));
        self
    }

    /// Creates a randomizer with standard variation.
    ///
    /// Uses speed range [0.7, 1.3] and volume range [0.6, 1.0].
    #[must_use]
    pub fn standard() -> Self {
        Self {
            volume_range: Some((0.6, 1.0)),
            speed_range: Some((0.7, 1.3)),
        }
    }

    /// Applies randomization to the given playback settings.
    pub fn apply(&self, settings: &mut PlaybackSettings) {
        let mut rng = rand::rng();

        if let Some((min, max)) = self.volume_range {
            settings.volume = Volume::Linear(rng.random_range(min..=max));
        }

        if let Some((min, max)) = self.speed_range {
            settings.speed = rng.random_range(min..=max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_concurrent_new() {
        let handle = Handle::default();
        let mc = MaxConcurrent::new(handle.clone(), 5);

        assert_eq!(mc.max, 5);
    }

    #[test]
    fn sound_effect_counter_with_interval() {
        let counter = SoundEffectCounter::with_interval(0.5);

        assert!(counter.counts.is_empty());
        assert_eq!(counter.timer.duration().as_secs_f32(), 0.5);
        assert_eq!(counter.timer.mode(), TimerMode::Repeating);
    }

    #[test]
    fn playback_randomizer_standard() {
        let randomizer = PlaybackRandomizer::standard();

        assert_eq!(randomizer.volume_range, Some((0.6, 1.0)));
        assert_eq!(randomizer.speed_range, Some((0.7, 1.3)));
    }

    #[test]
    fn playback_randomizer_builder() {
        let randomizer = PlaybackRandomizer::new()
            .with_volume(0.5, 0.9)
            .with_speed(0.8, 1.2);

        assert_eq!(randomizer.volume_range, Some((0.5, 0.9)));
        assert_eq!(randomizer.speed_range, Some((0.8, 1.2)));
    }

    #[test]
    fn playback_randomizer_applies_to_settings() {
        let randomizer = PlaybackRandomizer::new().with_volume(0.5, 0.5); // Fixed value for testing

        let mut settings = PlaybackSettings::default();
        randomizer.apply(&mut settings);

        match settings.volume {
            Volume::Linear(v) => assert!((v - 0.5).abs() < f32::EPSILON),
            _ => panic!("Expected linear volume"),
        }
    }
}
