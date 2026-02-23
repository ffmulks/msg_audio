//! # msg_audio
//!
//! A flexible audio management crate for Bevy games with:
//! - Volume control with master and category-based levels
//! - Concurrency limiting to prevent audio spam
//! - Pluggable audio category types (music and sound effects)
//! - Both component-based and event-based APIs
//!
//! ## Quick Start
//!
//! 1. Define your audio categories:
//!
//! ```rust
//! use bevy::prelude::*;
//! use msg_audio::{AudioCategory, MusicCategory, SfxCategory, AudioConfigTrait};
//!
//! // Music categories
//! #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
//! #[reflect(Component)]
//! pub enum GameMusic {
//!     MainMenu,
//!     #[default]
//!     Gameplay,
//! }
//!
//! // Sound effect categories
//! #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
//! #[reflect(Component)]
//! pub enum GameSfx {
//!     #[default]
//!     UI,
//!     Gameplay,
//!     Ambience,
//! }
//!
//! // Audio configuration
//! #[derive(Resource, Clone, Default, Reflect)]
//! #[reflect(Resource)]
//! pub struct GameAudioConfig {
//!     pub master: f32,
//!     pub main_menu_music: f32,
//!     pub gameplay_music: f32,
//!     pub ui_sfx: f32,
//!     pub gameplay_sfx: f32,
//!     pub ambience_sfx: f32,
//! }
//!
//! impl AudioConfigTrait for GameAudioConfig {
//!     fn master_volume(&self) -> f32 { self.master }
//! }
//!
//! impl AudioCategory for GameMusic {
//!     type Config = GameAudioConfig;
//!     fn volume_multiplier(&self, config: &Self::Config) -> f32 {
//!         match self {
//!             GameMusic::MainMenu => config.main_menu_music,
//!             GameMusic::Gameplay => config.gameplay_music,
//!         }
//!     }
//! }
//! impl MusicCategory for GameMusic {}
//!
//! impl AudioCategory for GameSfx {
//!     type Config = GameAudioConfig;
//!     fn volume_multiplier(&self, config: &Self::Config) -> f32 {
//!         match self {
//!             GameSfx::UI => config.ui_sfx,
//!             GameSfx::Gameplay => config.gameplay_sfx,
//!             GameSfx::Ambience => config.ambience_sfx,
//!         }
//!     }
//! }
//! impl SfxCategory for GameSfx {}
//! # fn main() {}
//! ```
//!
//! 2. Add the audio plugin:
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use msg_audio::prelude::*;
//! # #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
//! # #[reflect(Component)]
//! # enum GameMusic { #[default] Gameplay, MainMenu }
//! # #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
//! # #[reflect(Component)]
//! # enum GameSfx { #[default] UI }
//! # #[derive(Resource, Clone, Default, Reflect)]
//! # #[reflect(Resource)]
//! # struct GameAudioConfig { master: f32 }
//! # impl AudioConfigTrait for GameAudioConfig { fn master_volume(&self) -> f32 { self.master } }
//! # impl AudioCategory for GameMusic { type Config = GameAudioConfig; fn volume_multiplier(&self, _: &GameAudioConfig) -> f32 { 1.0 } }
//! # impl MusicCategory for GameMusic {}
//! # impl AudioCategory for GameSfx { type Config = GameAudioConfig; fn volume_multiplier(&self, _: &GameAudioConfig) -> f32 { 1.0 } }
//! # impl SfxCategory for GameSfx {}
//! # let mut app = App::new();
//! # app.add_plugins(MinimalPlugins);
//! # app.init_resource::<GameAudioConfig>();
//! app.add_plugins(MsgAudioPlugin::<GameMusic, GameSfx, GameAudioConfig>::default());
//! ```
//!
//! 3. Play audio:
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use msg_audio::prelude::*;
//! # #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
//! # #[reflect(Component)]
//! # enum GameMusic { #[default] Gameplay }
//! # #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
//! # #[reflect(Component)]
//! # enum GameSfx { #[default] UI, Gameplay }
//! # #[derive(Resource, Clone, Default)]
//! # struct Cfg;
//! # impl AudioConfigTrait for Cfg { fn master_volume(&self) -> f32 { 1.0 } }
//! # impl AudioCategory for GameMusic { type Config = Cfg; fn volume_multiplier(&self, _: &Cfg) -> f32 { 1.0 } }
//! # impl MusicCategory for GameMusic {}
//! # impl AudioCategory for GameSfx { type Config = Cfg; fn volume_multiplier(&self, _: &Cfg) -> f32 { 1.0 } }
//! # impl SfxCategory for GameSfx {}
//! # fn example(mut commands: Commands, mut messages: MessageWriter<PlaySfx<GameSfx>>, music_handle: Handle<AudioSource>, sfx_handle: Handle<AudioSource>) {
//! // Component-based (directly spawn)
//! commands.spawn(MusicBundle::new(music_handle, GameMusic::Gameplay));
//! commands.spawn(SfxBundle::new(sfx_handle.clone(), GameSfx::UI).randomized());
//!
//! // Message-based
//! messages.write(PlaySfx::new(sfx_handle, GameSfx::Gameplay));
//! # }
//! # fn main() {}
//! ```
//!
//! ## Features
//!
//! - **Pluggable Categories**: Define your own music and SFX category enums
//! - **Volume Management**: Automatic volume application based on master + category
//! - **Concurrency Limiting**: Prevent audio spam with per-sound limits
//! - **Randomization**: Built-in volume and pitch randomization for variety
//! - **Spatial Audio**: Positional audio with distance attenuation and entity attachment
//! - **Dual API**: Use component bundles or messages based on your needs

mod bundles;
mod components;
mod events;
mod systems;
mod traits;

pub use bundles::{MusicBundle, SfxBundle, DEFAULT_CONCURRENCY_INTERVAL, DEFAULT_MAX_CONCURRENT};
pub use components::{FadeOut, MaxConcurrent, PlaybackRandomizer, SoundEffectCounter};
#[cfg(feature = "spatial")]
pub use components::{AudioPlayerOf, AudioPlayers};
pub use events::{FadeOutMusic, PlayMusic, PlaySfx, StopAllMusic, StopMusic};
pub use traits::{AudioCategory, AudioConfigTrait, MusicCategory, SfxCategory};

use bevy::prelude::*;
#[cfg(feature = "spatial")]
use bevy::audio::{DefaultSpatialScale, SpatialScale};

/// Main plugin for the msg_audio crate.
///
/// This plugin sets up all the systems needed for audio management including:
/// - Volume application to new audio entities
/// - Volume updates when configuration changes
/// - Concurrency limiting for sound effects
/// - Event handling for play requests
/// - Spatial audio support (with the `spatial` feature)
///
/// # Type Parameters
///
/// - `M`: Your music category type implementing [`MusicCategory`]
/// - `S`: Your sound effect category type implementing [`SfxCategory`]
/// - `C`: Your audio config type implementing [`AudioConfigTrait`]
///
/// # Spatial Audio
///
/// When the `spatial` feature is enabled (default), you can configure a spatial scale
/// and use `.with_spatial()` or `.with_parent()` on audio messages and bundles.
///
/// **Important:** You must add a `SpatialListener` component to your listener entity
/// (e.g., camera or player) for spatial audio to work.
///
/// # Example
///
/// ```rust
/// # use bevy::prelude::*;
/// # use msg_audio::prelude::*;
/// # #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
/// # #[reflect(Component)]
/// # enum GameMusic { #[default] Gameplay }
/// # #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
/// # #[reflect(Component)]
/// # enum GameSfx { #[default] UI }
/// # #[derive(Resource, Clone, Default, Reflect)]
/// # #[reflect(Resource)]
/// # struct GameAudioConfig { master: f32 }
/// # impl AudioConfigTrait for GameAudioConfig { fn master_volume(&self) -> f32 { self.master } }
/// # impl AudioCategory for GameMusic { type Config = GameAudioConfig; fn volume_multiplier(&self, _: &GameAudioConfig) -> f32 { 1.0 } }
/// # impl MusicCategory for GameMusic {}
/// # impl AudioCategory for GameSfx { type Config = GameAudioConfig; fn volume_multiplier(&self, _: &GameAudioConfig) -> f32 { 1.0 } }
/// # impl SfxCategory for GameSfx {}
/// # let mut app = App::new();
/// # app.add_plugins(MinimalPlugins);
/// # app.init_resource::<GameAudioConfig>();
/// // Basic setup
/// app.add_plugins(MsgAudioPlugin::<GameMusic, GameSfx, GameAudioConfig>::default());
/// ```
pub struct MsgAudioPlugin<M, S, C>
where
    M: MusicCategory<Config = C>,
    S: SfxCategory<Config = C>,
    C: AudioConfigTrait,
{
    _phantom: std::marker::PhantomData<(M, S, C)>,
    #[cfg(feature = "spatial")]
    spatial_scale: Option<SpatialScale>,
}

impl<M, S, C> Default for MsgAudioPlugin<M, S, C>
where
    M: MusicCategory<Config = C>,
    S: SfxCategory<Config = C>,
    C: AudioConfigTrait,
{
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            #[cfg(feature = "spatial")]
            spatial_scale: None,
        }
    }
}

impl<M, S, C> MsgAudioPlugin<M, S, C>
where
    M: MusicCategory<Config = C>,
    S: SfxCategory<Config = C>,
    C: AudioConfigTrait,
{
    /// Sets the spatial audio scale for distance attenuation.
    ///
    /// For 2D games, use `SpatialScale::new_2d(1.0 / pixels_per_unit)`.
    /// For example, `SpatialScale::new_2d(1.0 / 100.0)` means 100 pixels = 1 audio unit.
    ///
    /// **Important:** You must add a `SpatialListener` component to your listener entity
    /// (e.g., camera or player) for spatial audio to work.
    #[cfg(feature = "spatial")]
    #[must_use]
    pub fn with_spatial_scale(mut self, scale: SpatialScale) -> Self {
        self.spatial_scale = Some(scale);
        self
    }
}

impl<M, S, C> Plugin for MsgAudioPlugin<M, S, C>
where
    M: MusicCategory<Config = C>,
    S: SfxCategory<Config = C>,
    C: AudioConfigTrait,
{
    fn build(&self, app: &mut App) {
        // Register types
        app.register_type::<MaxConcurrent>();
        app.register_type::<SoundEffectCounter>();
        app.register_type::<FadeOut>();

        // Initialize resources
        app.init_resource::<SoundEffectCounter>();

        // Spatial audio setup
        #[cfg(feature = "spatial")]
        if let Some(scale) = self.spatial_scale {
            app.insert_resource(DefaultSpatialScale(scale));
        }

        // Add messages (renamed from events in Bevy 0.17)
        app.add_message::<PlayMusic<M>>();
        app.add_message::<PlaySfx<S>>();
        app.add_message::<StopMusic<M>>();
        app.add_message::<StopAllMusic<M>>();
        app.add_message::<FadeOutMusic<M>>();

        // Add systems
        app.add_systems(
            Update,
            (
                // Apply volume to new audio
                systems::apply_volume_to_new_music::<M, C>,
                systems::apply_volume_to_new_sfx::<S, C>,
                // Update volume when config changes
                systems::update_music_volume::<M, C>.run_if(resource_changed::<C>),
                systems::update_sfx_volume::<S, C>.run_if(resource_changed::<C>),
                // Concurrency limiting
                systems::enforce_sfx_concurrency::<S>,
                // Fade processing
                systems::process_fade_outs,
                // Event handling
                events::handle_play_music_events::<M>,
                events::handle_play_sfx_events::<S>,
                events::handle_stop_music_events::<M>,
                events::handle_stop_all_music_events::<M>,
                events::handle_fade_out_music_events::<M>,
            ),
        );
    }
}

/// Minimal plugin that only registers types and resources.
///
/// Use this when you want more control over system scheduling
/// or need to add systems manually.
///
/// # Example
///
/// ```rust
/// # use bevy::prelude::*;
/// # use msg_audio::prelude::*;
/// # #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
/// # #[reflect(Component)]
/// # enum MyMusic { #[default] Main }
/// # #[derive(Resource, Clone, Default)]
/// # struct MyConfig;
/// # impl AudioConfigTrait for MyConfig { fn master_volume(&self) -> f32 { 1.0 } }
/// # impl AudioCategory for MyMusic { type Config = MyConfig; fn volume_multiplier(&self, _: &MyConfig) -> f32 { 1.0 } }
/// # impl MusicCategory for MyMusic {}
/// # let mut app = App::new();
/// # app.add_plugins(MinimalPlugins);
/// app.add_plugins(MsgAudioMinimalPlugin);
/// app.add_systems(Update, msg_audio::audio_systems::apply_volume_to_new_music::<MyMusic, MyConfig>);
/// ```
pub struct MsgAudioMinimalPlugin;

impl Plugin for MsgAudioMinimalPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MaxConcurrent>();
        app.register_type::<SoundEffectCounter>();
        app.register_type::<FadeOut>();
        app.init_resource::<SoundEffectCounter>();
    }
}

/// Re-export of system functions for custom scheduling.
pub mod audio_systems {
    pub use crate::systems::{
        apply_volume_to_new_music, apply_volume_to_new_sfx, enforce_sfx_concurrency,
        process_fade_outs, update_music_volume, update_sfx_volume,
    };
}

/// Re-export of event handler functions for custom scheduling.
pub mod audio_events {
    pub use crate::events::{
        handle_fade_out_music_events, handle_play_music_events, handle_play_sfx_events,
        handle_stop_all_music_events, handle_stop_music_events,
    };
}

/// Prelude module for convenient imports.
///
/// Import with `use msg_audio::prelude::*;` for quick access to all commonly used types.
pub mod prelude {
    pub use crate::bundles::{MusicBundle, SfxBundle, DEFAULT_MAX_CONCURRENT};
    pub use crate::components::{FadeOut, MaxConcurrent, PlaybackRandomizer, SoundEffectCounter};
    pub use crate::events::{FadeOutMusic, PlayMusic, PlaySfx, StopAllMusic, StopMusic};
    pub use crate::traits::{AudioCategory, AudioConfigTrait, MusicCategory, SfxCategory};
    pub use crate::{MsgAudioMinimalPlugin, MsgAudioPlugin};

    #[cfg(feature = "spatial")]
    pub use crate::components::{AudioPlayerOf, AudioPlayers};
    #[cfg(feature = "spatial")]
    pub use bevy::audio::{SpatialListener, SpatialScale};
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

    #[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
    #[reflect(Component)]
    enum TestSfx {
        #[default]
        UI,
    }

    #[derive(Resource, Clone, Default, Reflect)]
    #[reflect(Resource)]
    struct TestConfig {
        master: f32,
        music: f32,
        sfx: f32,
    }

    impl AudioConfigTrait for TestConfig {
        fn master_volume(&self) -> f32 {
            self.master
        }
    }

    impl AudioCategory for TestMusic {
        type Config = TestConfig;
        fn volume_multiplier(&self, config: &Self::Config) -> f32 {
            config.music
        }
    }
    impl MusicCategory for TestMusic {}

    impl AudioCategory for TestSfx {
        type Config = TestConfig;
        fn volume_multiplier(&self, config: &Self::Config) -> f32 {
            config.sfx
        }
    }
    impl SfxCategory for TestSfx {}

    #[test]
    fn plugin_builds_without_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<TestConfig>();
        app.add_plugins(MsgAudioPlugin::<TestMusic, TestSfx, TestConfig>::default());
        app.update();
    }

    #[test]
    fn minimal_plugin_registers_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(MsgAudioMinimalPlugin);
        app.update();

        assert!(app.world().contains_resource::<SoundEffectCounter>());
    }

    #[test]
    fn volume_multiplier_calculation() {
        let config = TestConfig {
            master: 0.8,
            music: 0.5,
            sfx: 0.75,
        };

        let music_vol = TestMusic::Main.volume_multiplier(&config);
        let sfx_vol = TestSfx::UI.volume_multiplier(&config);

        assert!((music_vol - 0.5).abs() < f32::EPSILON);
        assert!((sfx_vol - 0.75).abs() < f32::EPSILON);

        // Final volume = master * category
        let final_music = config.master_volume() * music_vol;
        let final_sfx = config.master_volume() * sfx_vol;

        assert!((final_music - 0.4).abs() < f32::EPSILON);
        assert!((final_sfx - 0.6).abs() < f32::EPSILON);
    }
}
