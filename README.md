# dmg_audio

A flexible, type-safe audio management crate for [Bevy](https://bevyengine.org/) games.

## Features

- **Pluggable Audio Categories** - Define your own music and SFX category enums with per-category volume control
- **Volume Management** - Automatic volume application combining master volume with category-specific levels
- **Concurrency Limiting** - Prevent audio spam by limiting concurrent instances of the same sound
- **Playback Randomization** - Built-in volume and pitch randomization for sound variety
- **Dual API** - Choose between component-based (bundles) or event-based (fire-and-forget) audio playback
- **Minimal Plugin Option** - Full control over system scheduling when needed

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
dmg_audio = "0.1"
```

With serde support for configuration serialization:

```toml
[dependencies]
dmg_audio = { version = "0.1", features = ["serde"] }
```

## Quick Start

### 1. Define Your Audio Categories

```rust
use bevy::prelude::*;
use dmg_audio::{AudioCategory, MusicCategory, SfxCategory, AudioConfigTrait};

// Music categories for different game states
#[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum GameMusic {
    #[default]
    MainMenu,
    Gameplay,
    Combat,
}

// Sound effect categories
#[derive(Component, Clone, Copy, Default, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum GameSfx {
    #[default]
    UI,
    Player,
    Environment,
}

// Your audio configuration resource
#[derive(Resource, Clone, Default, Reflect)]
#[reflect(Resource)]
pub struct GameAudioConfig {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
}

impl AudioConfigTrait for GameAudioConfig {
    fn master_volume(&self) -> f32 {
        self.master
    }
}

impl AudioCategory for GameMusic {
    type Config = GameAudioConfig;

    fn volume_multiplier(&self, config: &Self::Config) -> f32 {
        config.music
    }
}
impl MusicCategory for GameMusic {}

impl AudioCategory for GameSfx {
    type Config = GameAudioConfig;

    fn volume_multiplier(&self, config: &Self::Config) -> f32 {
        config.sfx
    }
}
impl SfxCategory for GameSfx {}
```

### 2. Add the Plugin

```rust
use dmg_audio::DmgAudioPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<GameAudioConfig>()
        .add_plugins(DmgAudioPlugin::<GameMusic, GameSfx, GameAudioConfig>::default())
        .run();
}
```

### 3. Play Audio

**Component-based (direct spawning):**

```rust
use dmg_audio::{MusicBundle, SfxBundle};

fn spawn_audio(mut commands: Commands, assets: Res<AssetServer>) {
    // Play background music (loops by default)
    commands.spawn(MusicBundle::new(
        assets.load("music/theme.ogg"),
        GameMusic::Gameplay,
    ));

    // Play a sound effect (despawns when finished)
    commands.spawn(SfxBundle::new(
        assets.load("sfx/click.ogg"),
        GameSfx::UI,
    ));

    // Sound effect with randomization
    commands.spawn(
        SfxBundle::new(assets.load("sfx/footstep.ogg"), GameSfx::Player)
            .randomized()
            .with_max_concurrent(3)
    );
}
```

**Event-based (fire-and-forget):**

```rust
use dmg_audio::{PlayMusic, PlaySfx, StopMusic, FadeOutMusic};
use std::time::Duration;

fn play_sound(mut sfx_events: EventWriter<PlaySfx<GameSfx>>, assets: Res<AssetServer>) {
    sfx_events.write(
        PlaySfx::new(assets.load("sfx/explosion.ogg"), GameSfx::Environment)
            .randomized()
            .with_max_concurrent(5)
    );
}

fn stop_music(mut events: EventWriter<StopMusic<GameMusic>>) {
    events.write(StopMusic::new(GameMusic::Combat));
}

fn fade_to_new_track(mut events: EventWriter<FadeOutMusic<GameMusic>>) {
    events.write(FadeOutMusic::from_secs(GameMusic::Combat, 2.0));
}
```

## API Overview

### Traits

| Trait | Purpose |
|-------|---------|
| `AudioCategory` | Base trait providing volume multiplier for a category |
| `MusicCategory` | Marker trait for music categories (typically looping) |
| `SfxCategory` | Marker trait for sound effect categories (typically one-shot) |
| `AudioConfigTrait` | Trait for your configuration resource providing master volume |

### Components

| Component | Purpose |
|-----------|---------|
| `MaxConcurrent` | Limits concurrent instances of a sound |
| `SoundEffectCounter` | Resource tracking active sound counts |
| `PlaybackRandomizer` | Builder for volume/pitch randomization |
| `FadeOut` | Gradual volume reduction with auto-despawn |

### Bundles

| Bundle | Purpose |
|--------|---------|
| `MusicBundle<M>` | Spawn music with looping playback |
| `SfxBundle<S>` | Spawn SFX with despawn-on-finish and concurrency limiting |

### Events

| Event | Purpose |
|-------|---------|
| `PlayMusic<M>` | Request music playback (fire-and-forget) |
| `PlaySfx<S>` | Request SFX playback (fire-and-forget) |
| `StopMusic<M>` | Stop music of a specific category |
| `StopAllMusic<M>` | Stop all currently playing music |
| `FadeOutMusic<M>` | Gradually fade out music over time |

### Plugins

| Plugin | Purpose |
|--------|---------|
| `DmgAudioPlugin<M, S, C>` | Full-featured plugin with all systems |
| `DmgAudioMinimalPlugin` | Minimal plugin for custom system scheduling |

## Advanced Usage

### Custom System Scheduling

Use `DmgAudioMinimalPlugin` for manual control:

```rust
use dmg_audio::{DmgAudioMinimalPlugin, audio_systems, audio_events};

app.add_plugins(DmgAudioMinimalPlugin);
app.add_systems(Update, (
    audio_systems::apply_volume_to_new_music::<GameMusic, GameAudioConfig>,
    audio_systems::apply_volume_to_new_sfx::<GameSfx, GameAudioConfig>,
    audio_events::handle_play_music_events::<GameMusic>,
    audio_events::handle_play_sfx_events::<GameSfx>,
));
```

### Per-Category Volume Control

Define different volume levels per category:

```rust
impl AudioCategory for GameSfx {
    type Config = GameAudioConfig;

    fn volume_multiplier(&self, config: &Self::Config) -> f32 {
        match self {
            GameSfx::UI => config.ui_volume,
            GameSfx::Player => config.player_volume,
            GameSfx::Environment => config.environment_volume,
        }
    }
}
```

### Playback Randomization

Add variety to repeated sounds:

```rust
// Standard randomization (speed 0.7-1.3, volume 0.6-1.0)
SfxBundle::new(handle, category).randomized()

// Custom ranges
SfxBundle::new(handle, category)
    .with_volume(0.8, 1.0)
    .with_speed(0.9, 1.1)

// Via PlaybackRandomizer directly
let mut settings = PlaybackSettings::DESPAWN;
PlaybackRandomizer::new()
    .with_volume(0.5, 1.0)
    .with_speed(0.8, 1.2)
    .apply(&mut settings);
```

### Concurrency Limiting

Prevent audio spam from rapid sound triggers:

```rust
// Limit footsteps to 3 concurrent instances
SfxBundle::new(footstep_handle, GameSfx::Player)
    .with_max_concurrent(3)

// Default is 5 concurrent instances (DEFAULT_MAX_CONCURRENT)
```

### Global Mute

Implement `is_muted()` in your config to support global audio muting:

```rust
impl AudioConfigTrait for GameAudioConfig {
    fn master_volume(&self) -> f32 {
        self.master
    }

    fn is_muted(&self) -> bool {
        self.muted
    }
}

// Toggle mute in your system
fn toggle_mute(mut config: ResMut<GameAudioConfig>) {
    config.muted = !config.muted;
}
```

When muted, all audio plays at zero volume but continues to run (useful for keeping music position).

## Volume Calculation

Final volume is calculated as:

```
final_volume = master_volume * category_volume * playback_volume
```

Where:
- `master_volume` - From `AudioConfigTrait::master_volume()`
- `category_volume` - From `AudioCategory::volume_multiplier()`
- `playback_volume` - From `PlaybackSettings::volume` (supports randomization)

## Bevy Compatibility

| dmg_audio | Bevy |
|-----------|------|
| 0.1       | 0.16 |

## Build Requirements

On Linux, you need the following system libraries:

```bash
# Debian/Ubuntu
sudo apt-get install libasound2-dev libudev-dev

# Fedora
sudo dnf install alsa-lib-devel systemd-devel

# Arch
sudo pacman -S alsa-lib systemd-libs
```

On macOS and Windows, no additional system libraries are required.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
