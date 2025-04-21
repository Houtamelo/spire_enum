#![allow(unused)]
use super::*;

pub trait Setting {
    fn key(&self) -> &'static str;

    fn apply(&self) {}

    fn on_confirm(&self) {}
}

#[delegated_enum(generate_variants(derive(Debug, Clone, Copy)), impl_conversions)]
#[derive(Clone, Copy)]
pub enum SettingsEnum {
    #[dont_generate_type]
    SpireWindowMode(SpireWindowMode),
    #[dont_generate_type]
    SkillOverlayMode(SkillOverlayMode),
    MaxFps(i32),
    DialogueTextSpeed(i32),
    Vsync(bool),
    MainVolume(i32),
    MusicVolume(i32),
    SfxVolume(i32),
    VoiceVolume(i32),
}

#[delegate_impl]
impl Setting for SettingsEnum {
    fn key(&self) -> &'static str;
    fn apply(&self);
    fn on_confirm(&self);
}

impl Setting for MaxFps {
    fn key(&self) -> &'static str {
        "max_fps"
    }
}

impl Setting for DialogueTextSpeed {
    fn key(&self) -> &'static str {
        "dialogue_text_speed"
    }
}

impl Setting for Vsync {
    fn key(&self) -> &'static str {
        "vsync"
    }
}

impl Setting for MainVolume {
    fn key(&self) -> &'static str {
        "main_volume"
    }
}

impl Setting for MusicVolume {
    fn key(&self) -> &'static str {
        "music_volume"
    }
}

impl Setting for SfxVolume {
    fn key(&self) -> &'static str {
        "sfx_volume"
    }
}

impl Setting for VoiceVolume {
    fn key(&self) -> &'static str {
        "voice_volume"
    }
}

macro_rules! impl_defaults {
    ($( $T: ty = $D: expr ),* $(,)? ) => {
	    $(
		    impl Default for $T {
			    fn default() -> Self { Self($D) }
		    }
	    )*
    };
}

impl_defaults! {
    MaxFps = 60,
    DialogueTextSpeed = 100,
    Vsync = true,
    MainVolume = 50,
    MusicVolume = 50,
    SfxVolume = 50,
    VoiceVolume = 50,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum SpireWindowMode {
    Windowed = 0,
    #[default]
    Maximized = 2,
    Fullscreen = 3,
    ExclusiveFullscreen = 4,
}

// Pending Translation Hook
impl SpireWindowMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::Maximized => "Maximized",
            Self::Fullscreen => "Fullscreen",
            Self::ExclusiveFullscreen => "Exclusive Fullscreen",
        }
    }

    pub fn index(&self) -> i32 {
        match self {
            Self::Windowed => 0,
            Self::Maximized => 1,
            Self::Fullscreen => 2,
            Self::ExclusiveFullscreen => 3,
        }
    }
}

impl Setting for SpireWindowMode {
    fn key(&self) -> &'static str {
        "window_mode"
    }
}

const DISPLAY_SKILL_OVERLAY_MODE_AUTO: &str = "Auto";

const DISPLAY_SKILL_OVERLAY_MODE_WAIT: &str = "Wait for Input";

#[derive(Clone, Copy, Debug)]
pub enum SkillOverlayMode {
    Auto { delay_ms: i64 },
    WaitForInput,
}

impl Default for SkillOverlayMode {
    fn default() -> Self {
        SkillOverlayMode::Auto { delay_ms: 3000 }
    }
}

// Pending Translation Hook
impl SkillOverlayMode {
    pub const VARIANT_DISPLAY_NAMES: &'static [&'static str] = &[
        SkillOverlayMode::Auto { delay_ms: 3000 }.display_name(),
        SkillOverlayMode::WaitForInput.display_name(),
    ];

    pub const fn display_name(&self) -> &'static str {
        match self {
            SkillOverlayMode::Auto { .. } => DISPLAY_SKILL_OVERLAY_MODE_AUTO,
            SkillOverlayMode::WaitForInput => DISPLAY_SKILL_OVERLAY_MODE_WAIT,
        }
    }

    pub const fn option_index(&self) -> i32 {
        match self {
            SkillOverlayMode::Auto { .. } => 0,
            SkillOverlayMode::WaitForInput => 1,
        }
    }
}

impl Setting for SkillOverlayMode {
    fn key(&self) -> &'static str {
        "skill_overlay_mode"
    }
}
