//! Audio systems for volume management and concurrency limiting.

use bevy::{audio::Volume, platform::collections::HashMap, prelude::*};

use crate::components::{MaxConcurrent, SoundEffectCounter};
use crate::traits::{AudioConfigTrait, MusicCategory, SfxCategory};

/// Applies volume settings to newly spawned music entities.
///
/// This system runs on `Added<AudioSink>` to apply the correct volume
/// based on the music category, master volume, and mute state.
pub fn apply_volume_to_new_music<M, C>(
    config: Res<C>,
    mut query: Query<(&M, &PlaybackSettings, &mut AudioSink), Added<AudioSink>>,
) where
    M: MusicCategory<Config = C>,
    C: AudioConfigTrait,
{
    for (category, playback, mut sink) in &mut query {
        let category_volume = category.volume_multiplier(&config);
        let playback_volume = extract_linear_volume(playback.volume);
        let final_volume = config.effective_volume() * category_volume * playback_volume;
        sink.set_volume(Volume::Linear(final_volume));
    }
}

/// Applies volume settings to newly spawned sound effect entities.
///
/// This system runs on `Added<AudioSink>` to apply the correct volume
/// based on the sound effect category, master volume, and mute state.
pub fn apply_volume_to_new_sfx<S, C>(
    config: Res<C>,
    mut query: Query<(&S, &PlaybackSettings, &mut AudioSink), Added<AudioSink>>,
) where
    S: SfxCategory<Config = C>,
    C: AudioConfigTrait,
{
    for (category, playback, mut sink) in &mut query {
        let category_volume = category.volume_multiplier(&config);
        let playback_volume = extract_linear_volume(playback.volume);
        let final_volume = config.effective_volume() * category_volume * playback_volume;
        sink.set_volume(Volume::Linear(final_volume));
    }
}

/// Updates volume on all active music entities when config changes.
///
/// This system should be run with `run_if(resource_changed::<C>)`.
/// Respects the mute state via [`AudioConfigTrait::effective_volume`].
pub fn update_music_volume<M, C>(
    config: Res<C>,
    mut query: Query<(&M, &PlaybackSettings, &mut AudioSink)>,
) where
    M: MusicCategory<Config = C>,
    C: AudioConfigTrait,
{
    for (category, playback, mut sink) in &mut query {
        let category_volume = category.volume_multiplier(&config);
        let playback_volume = extract_linear_volume(playback.volume);
        let final_volume = config.effective_volume() * category_volume * playback_volume;
        sink.set_volume(Volume::Linear(final_volume));
    }
}

/// Updates volume on all active sound effect entities when config changes.
///
/// This system should be run with `run_if(resource_changed::<C>)`.
/// Respects the mute state via [`AudioConfigTrait::effective_volume`].
pub fn update_sfx_volume<S, C>(
    config: Res<C>,
    mut query: Query<(&S, &PlaybackSettings, &mut AudioSink)>,
) where
    S: SfxCategory<Config = C>,
    C: AudioConfigTrait,
{
    for (category, playback, mut sink) in &mut query {
        let category_volume = category.volume_multiplier(&config);
        let playback_volume = extract_linear_volume(playback.volume);
        let final_volume = config.effective_volume() * category_volume * playback_volume;
        sink.set_volume(Volume::Linear(final_volume));
    }
}

/// Enforces maximum concurrent sound effect instances.
///
/// This system periodically resets counts and despawns excess sounds
/// to prevent audio spam.
pub fn enforce_sfx_concurrency<S: SfxCategory>(
    mut commands: Commands,
    time: Res<Time>,
    mut counter: ResMut<SoundEffectCounter>,
    query: Query<(Entity, &AudioPlayer, &MaxConcurrent), With<S>>,
) {
    // Reset counts periodically to prevent stale data
    if counter.timer.tick(time.delta()).just_finished() {
        counter.counts.clear();
    }

    // Track and limit concurrent sounds
    let mut kept_counts: HashMap<Handle<AudioSource>, u32> = HashMap::new();
    for (entity, audio_player, max) in &query {
        let kept_so_far = kept_counts.entry(audio_player.0.clone()).or_insert(0);
        if *kept_so_far >= max.max {
            commands.entity(entity).despawn();
        } else {
            *kept_so_far += 1;
        }
    }
}

/// Processes audio fade-outs.
///
/// This system updates the volume of entities with [`FadeOut`](crate::components::FadeOut)
/// components, gradually reducing volume and despawning when complete.
pub fn process_fade_outs(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut crate::components::FadeOut, &mut AudioSink)>,
) {
    for (entity, mut fade, mut sink) in &mut query {
        fade.timer.tick(time.delta());

        if fade.is_finished() {
            commands.entity(entity).despawn();
        } else {
            sink.set_volume(Volume::Linear(fade.current_volume()));
        }
    }
}

/// Extracts linear volume from a Volume enum.
///
/// Converts decibel values to linear using the formula: 10^(db/20)
#[inline]
fn extract_linear_volume(volume: Volume) -> f32 {
    match volume {
        Volume::Linear(v) => v,
        Volume::Decibels(db) => 10_f32.powf(db / 20.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_linear_volume_from_linear() {
        let volume = Volume::Linear(0.5);
        assert!((extract_linear_volume(volume) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn extract_linear_volume_from_decibels() {
        // -20dB should be approximately 0.1
        let volume = Volume::Decibels(-20.0);
        let linear = extract_linear_volume(volume);
        assert!((linear - 0.1).abs() < 0.001);
    }

    #[test]
    fn extract_linear_volume_zero_db() {
        // 0dB should be 1.0
        let volume = Volume::Decibels(0.0);
        let linear = extract_linear_volume(volume);
        assert!((linear - 1.0).abs() < f32::EPSILON);
    }
}
