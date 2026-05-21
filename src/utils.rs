use crate::core::models::RepeatMode;

pub const APP_NAME: &str = "hextune";

pub const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg"];

pub const TICK_RATE_MS: u64 = 16; // ~60 FPS event loop
pub const PROGRESS_BAR_WIDTH: usize = 40; // terminal progress bar chars
pub const EVENT_CHANNEL_CAPACITY: usize = 100;
pub const MIN_TRUNCATE_TITLE: usize = 8; // TUI list item min title width
pub const MIN_TRUNCATE_FIELD: usize = 4; // TUI list item min field width
pub const VOLUME_MAX: u8 = 100;
pub const VOLUME_STEP: u8 = 5;
pub const VOLUME_CURVE_EXPONENT: i32 = 4;
pub const CLI_PLAYBACK_POLL_MS: u64 = 100;

pub const SCAN_PROGRESS_INTERVAL: usize = 25;

pub const PREV_RESTART_THRESHOLD: f64 = 0.05;

/// Convert user volume percentage (0-100) to amplitude multiplier using perceptual scaling
///
/// Human hearing is logarithmic, so we use x^4 to approximate an exponential curve.
/// This provides a 60dB dynamic range, making volume changes feel more linear to human perception.
/// The difference between 1-2% will feel the same as the difference between 99-100%.
///
/// # Arguments
/// *`percent` - User input volume percentage (0-100)
///
/// # Returns
/// * `f32` - Amplitude multiplier (0.0-1.0)
pub fn volume_percent_to_amplitude(percent: u8) -> f32 {
    let x = (percent as f32) / (VOLUME_MAX as f32);
    x.powi(VOLUME_CURVE_EXPONENT)
}

/// Convert amplitude multiplier (0.0-1.0) back to user volume percentage (0-100)
/// This is the inverse of volume_percent_to_amplitude().
/// It converts the stored logarithmic amplitude back to linear percentage for display.
/// # Arguments
/// *
/// `amplitude` - Stored amplitude multiplier (0.0-1.0)
/// # Returns
/// *`u8` User volume percentage (0-100)
pub fn amplitude_to_volume (amplitude: f32) -> u8 {
    let x = amplitude.powf(1.0 / (VOLUME_CURVE_EXPONENT as f32));
    (x * (VOLUME_MAX as f32)).round() as u8
}

/// Separators used to split multiple artists in a raw tag string.
///
/// Ordered from most specific (word-boundary patterns) to least specific
/// (single characters) so that "Black/White feat. Someone" is split at
/// "feat." before "/" — preventing a fragment like "Black" being produced
/// from what was really a compound title token.
const SEPARATORS: &[&str] = &[" feat. ", " ft. ", " & ", "/", ";"];

/// Parse a raw artist tag string into individual, trimmed artist names.
///
/// Returns an empty `Vec` when `raw` is blank or whitespace-only.
pub fn parse_artists(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    // Iteratively split through each separator, accumulating fragments.
    let mut parts = vec![trimmed.to_owned()];

    for sep in SEPARATORS {
        parts = parts
            .into_iter()
            .flat_map(|p| {
                p.split(sep)
                    .map(str::trim)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .filter(|s| !s.is_empty())
            .collect();
    }

    parts
}

/// Format a slice of artist names for display.
///
/// Returns `"Unknown Artist"` when the slice is empty so callers never
/// have to special-case the empty case themselves.
pub fn format_artists(artists: &[String]) -> String {
    if artists.is_empty() {
        "Unknown Artist".to_owned()
    } else {
        artists.join(", ")
    }
}

pub fn repeat_label(mode: RepeatMode) -> &'static str {
    match mode {
        RepeatMode::Off => "Off",
        RepeatMode::All => "All",
        RepeatMode::One => "One",
    }
}