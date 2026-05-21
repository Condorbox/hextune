use crate::application::state::AppState;
use crate::core::events::UiEvent;
use crate::core::models::RepeatMode;
use crate::modules::input::InputAction;
use crate::utils::{amplitude_to_volume, PREV_RESTART_THRESHOLD_DEFAULT, PREV_RESTART_THRESHOLD_MIN, PREV_RESTART_THRESHOLD_STEP, VOLUME_MAX, VOLUME_STEP};

const SETTINGS_FIELDS: &[SettingsField] = &[
    SettingsField::Volume,
    SettingsField::Repeat,
    SettingsField::PrevRestartThreshold,
    SettingsField::MusicPath,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    MusicPath,
    Volume,
    Repeat,
    PrevRestartThreshold
}

/// Inline validation state for the path field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathValidation {
    /// User hasn't tried to confirm yet, or is still typing.
    Idle,
    /// Last confirm attempt failed
    Error(String),
}

/// Inline validation state for the previous-track restart threshold field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThresholdValidation {
    /// User hasn't tried to confirm yet, or is still typing.
    Idle,
    /// Last confirm attempt failed
    Error(String),
}


#[derive(Debug, Clone)]
pub struct SettingsState {
    open: bool,

    selected: SettingsField,

    editing_volume: bool,
    temp_volume: u8,

    temp_repeat: RepeatMode,

    editing_prev_threshold: bool,
    temp_prev_threshold: u8,
    threshold_validation: ThresholdValidation,

    editing_path: bool,
    temp_path: String,
    path_validation: PathValidation,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            open: false,
            selected: SettingsField::Volume,
            editing_volume: false,
            temp_volume: VOLUME_MAX,
            temp_repeat: RepeatMode::default(),
            editing_prev_threshold: false,
            temp_prev_threshold: PREV_RESTART_THRESHOLD_DEFAULT,
            threshold_validation: ThresholdValidation::Idle,
            editing_path: false,
            temp_path: String::new(),
            path_validation: PathValidation::Idle,
        }
    }
}

impl SettingsState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.editing_volume = false;
        self.editing_path = false;
        self.threshold_validation = ThresholdValidation::Idle;
        self.editing_prev_threshold = false;
        self.path_validation = PathValidation::Idle;
    }

    pub fn selected(&self) -> SettingsField {
        self.selected
    }

    pub fn is_editing_volume(&self) -> bool {
        self.editing_volume
    }

    pub fn is_editing_path(&self) -> bool {
        self.editing_path
    }

    pub fn is_editing_prev_threshold(&self) -> bool {
        self.editing_prev_threshold
    }

    pub fn threshold_validation(&self) -> &ThresholdValidation {
        &self.threshold_validation
    }

    pub fn temp_volume(&self) -> u8 {
        self.temp_volume
    }

    pub fn temp_repeat(&self) -> RepeatMode {
        self.temp_repeat
    }

    pub fn temp_prev_threshold(&self) -> u8 {
        self.temp_prev_threshold
    }

    pub fn temp_path(&self) -> &str {
        &self.temp_path
    }

    pub fn path_validation(&self) -> &PathValidation {
        &self.path_validation
    }

    pub fn sync_from_app_state(&mut self, app_state: &AppState) {
        self.temp_repeat = app_state.config.repeat;

        if !self.editing_path {
            self.temp_path = app_state
                .config
                .root_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
        }

        if !self.editing_volume || self.selected != SettingsField::Volume {
            self.temp_volume = amplitude_to_volume(app_state.config.volume);
        }

        if !self.editing_prev_threshold || self.selected != SettingsField::PrevRestartThreshold {
            self.temp_prev_threshold = app_state.config.prev_restart_threshold;
            self.threshold_validation = ThresholdValidation::Idle;
        }
    }

    pub fn apply_action(&mut self, action: InputAction) -> Vec<UiEvent> {
        let mut events = Vec::new();

        if self.editing_path {
            self.apply_path_action(action, &mut events);
            return events;
        }

        if self.editing_volume {
            self.apply_volume_action(action, &mut events);
            return events;
        }

        if self.editing_prev_threshold {
            self.apply_prev_threshold_action(action, &mut events);
            return events;
        }

        self.apply_navigation_action(action, &mut events);
        events
    }

    fn navigate_up(&mut self) {
        let current = SETTINGS_FIELDS
            .iter()
            .position(|f| *f == self.selected)
            .unwrap_or(0);
        let prev = (current + SETTINGS_FIELDS.len() - 1) % SETTINGS_FIELDS.len();
        self.selected = SETTINGS_FIELDS[prev];
    }

    fn navigate_down(&mut self) {
        let current = SETTINGS_FIELDS
            .iter()
            .position(|f| *f == self.selected)
            .unwrap_or(0);
        let next = (current + 1) % SETTINGS_FIELDS.len();
        self.selected = SETTINGS_FIELDS[next];
    }

    fn apply_volume_action(&mut self, action: InputAction, events: &mut Vec<UiEvent>) {
        match action {
            InputAction::SettingsConfirm => {
                self.editing_volume = false;
                events.push(UiEvent::VolumeChangeRequested {
                    volume: self.temp_volume,
                });
            }
            InputAction::SettingsClose => {
                self.editing_volume = false;
            }
            InputAction::SettingsLeft => {
                self.temp_volume = self.temp_volume.saturating_sub(VOLUME_STEP);
            }
            InputAction::SettingsRight => {
                self.temp_volume = self.temp_volume.saturating_add(VOLUME_STEP).min(VOLUME_MAX);
            }
            InputAction::SettingsTypeChar(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as u8;
                let new_val = (self.temp_volume % 10) * 10 + digit;
                if new_val <= VOLUME_MAX {
                    self.temp_volume = new_val;
                }
            }
            _ => {}
        }
    }

    fn apply_prev_threshold_action(&mut self, action: InputAction, events: &mut Vec<UiEvent>) {
        match action {
            InputAction::SettingsConfirm => {
                if self.temp_prev_threshold < PREV_RESTART_THRESHOLD_MIN {
                    // Keep editing and surface the error explicitly on confirm.
                    self.threshold_validation = ThresholdValidation::Error(format!(
                        "Minimum is {}%.",
                        PREV_RESTART_THRESHOLD_MIN
                    ));
                } else {
                    self.editing_prev_threshold = false;
                    self.threshold_validation = ThresholdValidation::Idle;
                    events.push(UiEvent::PrevThresholdSet {
                        threshold: self.temp_prev_threshold,
                    });
                }
            }
            InputAction::SettingsClose => {
                self.editing_prev_threshold = false;
                self.threshold_validation = ThresholdValidation::Idle;
            }
            InputAction::SettingsLeft => {
                self.temp_prev_threshold = self.temp_prev_threshold
                    .saturating_sub(PREV_RESTART_THRESHOLD_STEP)
                    .max(PREV_RESTART_THRESHOLD_MIN);
                self.threshold_validation = ThresholdValidation::Idle;
            }
            InputAction::SettingsRight => {
                self.temp_prev_threshold = self.temp_prev_threshold
                    .saturating_add(PREV_RESTART_THRESHOLD_STEP)
                    .min(100);
                self.threshold_validation = ThresholdValidation::Idle;
            }
            InputAction::SettingsTypeChar(c) if c.is_ascii_digit() => {
                let digit = c.to_digit(10).unwrap() as u8;
                let new_val = (self.temp_prev_threshold % 10) * 10 + digit;
                
                if new_val <= 100 {
                    self.temp_prev_threshold = new_val;
                    self.threshold_validation = if new_val < PREV_RESTART_THRESHOLD_MIN {
                        ThresholdValidation::Error(format!(
                            "Minimum is {}%.",
                            PREV_RESTART_THRESHOLD_MIN
                        ))
                    } else {
                        ThresholdValidation::Idle
                    };
                }
            }
            _ => {}
        }
    }

    fn apply_path_action(&mut self, action: InputAction, events: &mut Vec<UiEvent>) {
        match action {
            InputAction::SettingsConfirm => {
                let path = std::path::Path::new(&self.temp_path);
                if self.temp_path.is_empty() {
                    self.path_validation =
                        PathValidation::Error("Path cannot be empty.".to_string());
                } else if !path.exists() {
                    self.path_validation =
                        PathValidation::Error("Path does not exist.".to_string());
                } else if !path.is_dir() {
                    self.path_validation =
                        PathValidation::Error("Path is not a directory.".to_string());
                } else {
                    self.editing_path = false;
                    self.path_validation = PathValidation::Idle;
                    events.push(UiEvent::PathChangeRequested {
                        path: path.to_path_buf(),
                    });
                }
            }
            InputAction::SettingsClose => {
                self.editing_path = false;
                self.path_validation = PathValidation::Idle;
            }
            InputAction::SettingsClearLine => {
                self.temp_path.clear();
                self.path_validation = PathValidation::Idle;
            }
            InputAction::SettingsBackspace => {
                self.temp_path.pop();
                self.path_validation = PathValidation::Idle;
            }
            InputAction::SettingsTypeChar(c) => {
                self.temp_path.push(c);
                self.path_validation = PathValidation::Idle;
            }
            _ => {}
        }
    }

    fn apply_navigation_action(&mut self, action: InputAction, events: &mut Vec<UiEvent>) {
        match action {
            InputAction::SettingsClose => self.close(),
            InputAction::SettingsNavigateUp => {
                self.navigate_up();
            }
            InputAction::SettingsNavigateDown => {
                self.navigate_down();
            }
            InputAction::SettingsConfirm => match self.selected {
                SettingsField::Volume => {
                    self.editing_volume = true;
                }
                SettingsField::Repeat => {
                    self.temp_repeat = self.temp_repeat.cycle();
                    events.push(UiEvent::RepeatChangeRequested {
                        mode: self.temp_repeat,
                    });
                }
                SettingsField::MusicPath => {
                    self.editing_path = true;
                    self.path_validation = PathValidation::Idle;
                }
                SettingsField::PrevRestartThreshold => {
                    self.editing_prev_threshold = true;
                }
            },
            InputAction::SettingsLeft if self.selected == SettingsField::Repeat => {
                self.temp_repeat = self.temp_repeat.cycle_back();
                events.push(UiEvent::RepeatChangeRequested {
                    mode: self.temp_repeat,
                });
            }
            InputAction::SettingsRight if self.selected == SettingsField::Repeat => {
                self.temp_repeat = self.temp_repeat.cycle();
                events.push(UiEvent::RepeatChangeRequested {
                    mode: self.temp_repeat,
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::UiEvent;
    use crate::modules::input::InputAction;

    fn open_and_select_repeat(s: &mut SettingsState) {
        s.open();
        s.apply_action(InputAction::SettingsNavigateDown);
        assert_eq!(s.selected(), SettingsField::Repeat);
    }

    fn open_and_select_path(s: &mut SettingsState) {
        s.open();
        s.apply_action(InputAction::SettingsNavigateDown);
        s.apply_action(InputAction::SettingsNavigateDown);
        assert_eq!(s.selected(), SettingsField::MusicPath);
    }

    #[test]
    fn volume_edit_confirm_emits_event() {
        let mut s = SettingsState::default();
        s.open();

        let events = s.apply_action(InputAction::SettingsConfirm);
        assert!(events.is_empty());
        assert!(s.is_editing_volume());

        s.apply_action(InputAction::SettingsRight);
        let events = s.apply_action(InputAction::SettingsConfirm);

        assert!(!s.is_editing_volume());
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], UiEvent::VolumeChangeRequested { .. }));
    }

    #[test]
    fn volume_edit_close_cancels_without_emitting_and_keeps_modal_open() {
        let mut s = SettingsState::default();
        s.open();

        let events = s.apply_action(InputAction::SettingsConfirm);
        assert!(events.is_empty());
        assert!(s.is_editing_volume());

        let events = s.apply_action(InputAction::SettingsClose);
        assert!(events.is_empty());
        assert!(s.is_open());
        assert!(!s.is_editing_volume());
    }

    #[test]
    fn volume_edit_digit_typing_and_saturation() {
        // Saturation at max.
        let mut s = SettingsState::default();
        s.open();
        s.apply_action(InputAction::SettingsConfirm);
        assert_eq!(s.temp_volume(), VOLUME_MAX);
        s.apply_action(InputAction::SettingsRight);
        assert_eq!(s.temp_volume(), VOLUME_MAX);
        s.apply_action(InputAction::SettingsLeft);
        assert_eq!(s.temp_volume(), VOLUME_MAX.saturating_sub(VOLUME_STEP));

        // Digit typing.
        let mut s = SettingsState::default();
        s.open();
        s.apply_action(InputAction::SettingsConfirm);
        s.apply_action(InputAction::SettingsTypeChar('5'));
        assert_eq!(s.temp_volume(), 5);
        s.apply_action(InputAction::SettingsTypeChar('0'));
        assert_eq!(s.temp_volume(), 50);
    }

    #[test]
    fn path_edit_empty_sets_error() {
        let mut s = SettingsState::default();
        open_and_select_path(&mut s);

        s.apply_action(InputAction::SettingsConfirm);
        assert!(s.is_editing_path());

        let events = s.apply_action(InputAction::SettingsConfirm);
        assert!(events.is_empty());
        assert!(matches!(s.path_validation(), PathValidation::Error(_)));
        assert!(s.is_editing_path());
    }

    #[test]
    fn path_edit_valid_dir_emits_event_and_exits_edit() {
        let mut s = SettingsState::default();
        open_and_select_path(&mut s);
        s.apply_action(InputAction::SettingsConfirm);

        let dir = std::env::temp_dir().join(format!(
            "music_cli_settings_state_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        for ch in dir.to_string_lossy().chars() {
            s.apply_action(InputAction::SettingsTypeChar(ch));
        }

        let events = s.apply_action(InputAction::SettingsConfirm);
        assert!(!s.is_editing_path());
        assert!(matches!(s.path_validation(), PathValidation::Idle));
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], UiEvent::PathChangeRequested { .. }));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ctrl_u_clears_path() {
        let mut s = SettingsState::default();
        open_and_select_path(&mut s);
        s.apply_action(InputAction::SettingsConfirm);

        s.apply_action(InputAction::SettingsTypeChar('a'));
        assert_eq!(s.temp_path(), "a");

        s.apply_action(InputAction::SettingsClearLine);
        assert_eq!(s.temp_path(), "");
    }

    #[test]
    fn path_edit_backspace_and_typing_clears_stale_error() {
        let mut s = SettingsState::default();
        open_and_select_path(&mut s);
        s.apply_action(InputAction::SettingsConfirm);

        // Force an error first.
        s.apply_action(InputAction::SettingsConfirm);
        assert!(matches!(s.path_validation(), PathValidation::Error(_)));

        s.apply_action(InputAction::SettingsTypeChar('a'));
        assert!(matches!(s.path_validation(), PathValidation::Idle));
        assert_eq!(s.temp_path(), "a");

        s.apply_action(InputAction::SettingsTypeChar('b'));
        assert_eq!(s.temp_path(), "ab");
        s.apply_action(InputAction::SettingsBackspace);
        assert_eq!(s.temp_path(), "a");
    }

    #[test]
    fn navigation_wraps_in_both_directions() {
        let mut s = SettingsState::default();
        s.open();
        assert_eq!(s.selected(), SettingsField::Volume);

        s.apply_action(InputAction::SettingsNavigateUp);
        assert_eq!(s.selected(), SettingsField::MusicPath);

        s.apply_action(InputAction::SettingsNavigateDown);
        assert_eq!(s.selected(), SettingsField::Volume);
    }

    #[test]
    fn repeat_cycles_forward_and_backward() {
        let mut s = SettingsState::default();
        open_and_select_repeat(&mut s);
        let start = s.temp_repeat();

        let events = s.apply_action(InputAction::SettingsConfirm);
        assert_eq!(s.temp_repeat(), start.cycle());
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], UiEvent::RepeatChangeRequested { .. }));

        let events = s.apply_action(InputAction::SettingsLeft);
        assert_eq!(s.temp_repeat(), start);
        assert_eq!(events.len(), 1);

        let events = s.apply_action(InputAction::SettingsRight);
        assert_eq!(s.temp_repeat(), start.cycle());
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn close_in_navigation_closes_modal_but_close_in_edit_exits_edit_only() {
        let mut s = SettingsState::default();
        s.open();
        assert!(s.is_open());

        s.apply_action(InputAction::SettingsConfirm);
        assert!(s.is_editing_volume());
        s.apply_action(InputAction::SettingsClose);
        assert!(s.is_open());
        assert!(!s.is_editing_volume());

        s.apply_action(InputAction::SettingsClose);
        assert!(!s.is_open());
    }
}
