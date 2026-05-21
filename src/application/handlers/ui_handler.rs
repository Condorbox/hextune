use crate::application::handlers::HandlerContext;
use crate::core::events::{AppEvent, LibraryEvent, PlaybackEvent, UiEvent};
use crate::core::models::RepeatMode;
use crate::utils::{volume_percent_to_amplitude, PREV_RESTART_THRESHOLD};
use anyhow::Result;
use crate::modules::library::sorter::SortField;

/// Handles all [`UiEvent`] variants that require side effects.
///
/// Responsible for:
/// - Translating user intent into domain events (play, next, prev, volume, shuffle, repeat).
/// - Validating input before acting (e.g. path must be a valid directory).
/// - Persisting config changes to storage.
///
/// Pure state updates (ShowMessage, ShowError, SelectionChanged, SearchToggled,
/// SearchQueryChanged) are already handled by `AppState::apply_event`.
pub struct UiHandler;

impl UiHandler {
    pub fn handle(&self, event: &UiEvent, ctx: &mut HandlerContext) -> Result<()> {
        match event {
            UiEvent::PlaySelectedRequested => {
                let song = {
                    let state = ctx.state.lock().unwrap();
                    state.ui.selected_index
                        .and_then(|i| state.library.songs.get(i).cloned())
                };
                if let Some(song) = song {
                    ctx.event_tx
                        .send(AppEvent::Playback(PlaybackEvent::PlayRequested { song }))?;
                }
            }

            UiEvent::TogglePauseRequested => {
                if let Some(playback) = ctx.playback.as_mut() {
                    if playback.is_paused() {
                        playback.resume();
                        ctx.event_tx
                            .send(AppEvent::Playback(PlaybackEvent::Resumed))?;
                    } else if playback.is_playing() {
                        playback.pause();
                        ctx.event_tx
                            .send(AppEvent::Playback(PlaybackEvent::Paused))?;
                    }
                }
            }

            UiEvent::NextTrackRequested => {
                // RepeatMode::One does not loop on manual nav — user explicitly wants to move.
                let (current_index, library_len, loop_playlist) = {
                    let state = ctx.state.lock().unwrap();
                    (
                        state.ui.selected_index,
                        state.library.songs.len(),
                        state.config.repeat == RepeatMode::All,
                    )
                };

                // Re-initialize shuffle queue if this pass ran dry.
                if ctx.shuffle_manager.is_enabled() && ctx.shuffle_manager.remaining_in_pass() == 0 {
                    ctx.shuffle_manager.initialize(library_len, current_index);
                }

                ctx.advance_to_next(current_index, library_len, loop_playlist)?;
            }

            UiEvent::PreviousTrackRequested => {
                // RepeatMode::One does not loop on manual nav — user explicitly wants to move.
                let (current_index, library_len, loop_playlist, should_restart) = {
                    let state = ctx.state.lock().unwrap();
                    let should_restart = state
                        .playback
                        .current_song
                        .as_ref()
                        .and_then(|s| s.duration)
                        .is_some_and(|d| {
                            !d.is_zero() &&
                                state.playback.current_elapsed > d.mul_f64(PREV_RESTART_THRESHOLD)
                        });

                    (
                        state.ui.selected_index,
                        state.library.songs.len(),
                        state.config.repeat == RepeatMode::All,
                        should_restart,
                    )
                };
                
                if should_restart {
                    // Re-play the current song from the beginning.
                    let song = ctx.state.lock().unwrap().playback.current_song.clone();
                    if let Some(song) = song {
                        ctx.event_tx
                            .send(AppEvent::Playback(PlaybackEvent::PlayRequested { song }))?;
                    }
                } else {
                    ctx.advance_to_prev(current_index, library_len, loop_playlist)?;
                }
            }

            UiEvent::VolumeChangeRequested { volume } => {
                let volume_f32 = volume_percent_to_amplitude(*volume);
                ctx.event_tx
                    .send(AppEvent::Playback(PlaybackEvent::VolumeChanged {
                        volume: volume_f32,
                    }))?;
                ctx.event_tx.send(AppEvent::Ui(UiEvent::ShowMessage {
                    message: format!("Volume set to {}%", volume),
                }))?;
            }

            UiEvent::PathChangeRequested { path } => {
                match path.canonicalize() {
                    Ok(canonical) if canonical.is_dir() => {
                        ctx.state.lock().unwrap().config.root_path = Some(canonical);
                        ctx.persist_state()?;
                        ctx.event_tx.send(AppEvent::Ui(UiEvent::ShowMessage {
                            message: "Music path updated. Run refresh to scan.".to_string(),
                        }))?;
                    }
                    Ok(_) => {
                        ctx.event_tx.send(AppEvent::Ui(UiEvent::ShowError {
                            message: "Path exists but is not a directory.".to_string(),
                        }))?;
                    }
                    Err(e) => {
                        ctx.event_tx.send(AppEvent::Ui(UiEvent::ShowError {
                            message: format!("Invalid path: {}", e),
                        }))?;
                    }
                }

            }

            UiEvent::SearchToggled { active } => {
                if !active {
                    ctx.event_tx.send(AppEvent::Ui(UiEvent::ShowMessage {
                        message: "Search cleared".to_string(),
                    }))?;
                }
            }

            UiEvent::SearchQueryChanged { query } => {
                ctx.event_tx
                    .send(AppEvent::Library(LibraryEvent::SearchRequested {
                        query: query.clone(),
                    }))?;
            }

            UiEvent::ShuffleToggled { shuffle_enabled } => {
                // `shuffle_enabled` is the *current* state — toggling means flipping it.
                Self::apply_shuffle(ctx, !shuffle_enabled)?;
            }

            UiEvent::ShuffleSet { enabled } => {
                Self::apply_shuffle(ctx, *enabled)?;
            }

            UiEvent::RepeatChangeRequested { mode } => {
                ctx.event_tx
                    .send(AppEvent::Playback(PlaybackEvent::RepeatChanged { mode: *mode }))?;
            }

            UiEvent::RefreshRequested => {
                let root_path = ctx.state.lock().unwrap().config.root_path.clone();
                match root_path {
                    Some(path) => {
                        ctx.event_tx
                            .send(AppEvent::Library(LibraryEvent::ScanRequested { path }))?;
                    }
                    None => {
                        ctx.event_tx.send(AppEvent::Ui(UiEvent::ShowError {
                            message: "No music path set. Configure it in Settings (s).".to_string(),
                        }))?;
                    }
                }
            }

            UiEvent::SortCycleRequested => {
                let next_field = {
                    let state = ctx.state.lock().unwrap();
                    match state.library.active_sort {
                        None => Some(SortField::default()),     // natural → title
                        Some(SortField::Duration) => None,      // duration → natural
                        Some(f) => Some(f.next()),     // title→artist→album→duration
                    }
                };
                ctx.event_tx
                    .send(AppEvent::Library(LibraryEvent::SortRequested { field: next_field }))?;
            }

            UiEvent::QuitRequested => {
                ctx.event_tx.send(AppEvent::Shutdown)?;
            }

            // Pure state updates — already handled by AppState::apply_event.
            UiEvent::ShowMessage { .. }
            | UiEvent::ShowError { .. }
            | UiEvent::SelectionChanged { .. } => {}
        }

        Ok(())
    }

    /// Applies a new shuffle state: updates the manager, initializes the queue
    /// if enabling, then emits the event so `apply_event` persists it to config.
    fn apply_shuffle(ctx: &mut HandlerContext, enabled: bool) -> Result<()> {
        ctx.shuffle_manager.set_enabled(enabled);

        if enabled {
            let (current_index, playlist_size) = {
                let state = ctx.state.lock().unwrap();
                (state.ui.selected_index, state.library.songs.len())
            };
            ctx.shuffle_manager.initialize(playlist_size, current_index);
        }

        ctx.event_tx
            .send(AppEvent::Playback(PlaybackEvent::Shuffle { enabled }))?;

        Ok(())
    }
}