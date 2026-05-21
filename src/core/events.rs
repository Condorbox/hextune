use crate::core::models::{RepeatMode, Song};
use std::path::PathBuf;
use crate::modules::library::sorter::SortField;

/// All events that can occur in the application
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Playback events
    Playback(PlaybackEvent),

    // Library events
    Library(LibraryEvent),

    // UI events
    Ui(UiEvent),

    // Application lifecycle
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum PlaybackEvent {
    /// Request to play a specific song
    PlayRequested { song: Song },

    /// Playback started
    Started { song: Song },

    /// Playback paused
    Paused,

    /// Playback resumed
    Resumed,

    /// Current track finished
    TrackFinished,

    /// Volume changed (0.0 - 1.0)
    VolumeChanged { volume: f32 },

    /// Shuffle enabled or disabled
    Shuffle { enabled: bool },

    /// Repeat mode changed.
    RepeatChanged { mode: RepeatMode },
}

#[derive(Debug, Clone)]
pub enum LibraryEvent {
    /// Request to scan directory
    ScanRequested { path: PathBuf },

    /// Scanning started
    ScanStarted { path: PathBuf },

    /// Scan progress update
    ScanProgress { found: usize },

    /// Scanning completed
    ScanCompleted { songs: Vec<Song>, count: usize },

    /// Scanning failed 
    ScanFailed { path: PathBuf, message: String },

    /// Library loaded from storage
    LibraryLoaded { songs: Vec<Song> },

    /// Search requested
    SearchRequested { query: String },

    /// Search results
    SearchResults { results: Vec<usize> },

    /// User requested a sort order change
    SortRequested { field: Option<SortField> },

    /// Sort has been applied. `library.songs` is already in the new order.
    /// Carries the re-anchored indices so `apply_event` can update state
    /// without needing to search the vec again.
    SortChanged {
        field: Option<SortField>,
        /// New position of the previously-selected song, if any.
        new_selected_index: Option<usize>,
        /// New position of the currently-playing song, if any.
        new_current_index: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    /// User requested to play selected song
    PlaySelectedRequested,

    /// User requested pause/resume toggle
    TogglePauseRequested,

    /// User requested next track
    NextTrackRequested,

    /// User requested previous track
    PreviousTrackRequested,

    /// User changed selection
    SelectionChanged { index: usize },

    /// User requested quit
    QuitRequested,

    /// Display message to user
    ShowMessage { message: String },

    /// Display error to user
    ShowError { message: String },

    /// User requested volume change (0-100)
    VolumeChangeRequested { volume: u8 },

    /// User requested path change
    PathChangeRequested { path: PathBuf },

    /// Search mode toggled
    SearchToggled { active: bool },

    /// Search query updated
    SearchQueryChanged { query: String },

    /// Shuffle toggled
    ShuffleToggled {shuffle_enabled: bool},

    /// Set shuffle state explicitly (not toggle)
    ShuffleSet { enabled: bool },

    /// Set repeat mode explicitly
    RepeatChangeRequested { mode: RepeatMode },

    /// User requested a library refresh
    RefreshRequested,

    /// User requested the sort field to advance to the next option
    SortCycleRequested,

    /// Set the previous-track restart threshold (5–100 percent)
    PrevThresholdSet { threshold: u8 },
}

/// Type alias for event sender
pub type EventSender = crossbeam_channel::Sender<AppEvent>;

/// Type alias for event receiver
pub type EventReceiver = crossbeam_channel::Receiver<AppEvent>;