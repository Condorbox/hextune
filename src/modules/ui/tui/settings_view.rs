use crate::modules::ui::tui::settings_state::{PathValidation, SettingsField, SettingsState, ThresholdValidation};
use crate::modules::input::{InputAction, InputMode, KeyConfig};
use crate::modules::ui::key_hints;
use crate::utils::repeat_label;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crossterm::event::KeyCode;

pub fn draw(f: &mut Frame, settings: &SettingsState, key_config: &KeyConfig) {
    let height_pct = if settings.is_editing_path() { 70 } else { 65 };
    let area = centered_rect(60, height_pct, f.area());
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .title(" ⚙ Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
        area,
    );

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(4),
    };

    let threshold_error_height = match settings.threshold_validation() {
        ThresholdValidation::Error(_) => 1,
        ThresholdValidation::Idle => 0,
    };

    let path_error_height = match settings.path_validation() {
        PathValidation::Error(_) => 1,
        PathValidation::Idle => 0,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                 // Volume
            Constraint::Length(3),                 // Repeat
            Constraint::Length(3),                 // Prev restart threshold
            Constraint::Length(threshold_error_height), // Threshold inline error (0 or 1)
            Constraint::Length(3),                 // Music Path input
            Constraint::Length(path_error_height), // Inline error (0 or 1)
            Constraint::Min(0),                    // spacer
            Constraint::Length(2),                 // help
        ])
        .split(inner);

    draw_volume(f, settings, key_config, chunks[0]);
    draw_repeat(f, settings, key_config, chunks[1]);
    draw_prev_threshold(f, settings, key_config, chunks[2]);
    draw_threshold_error(f, settings, chunks[3]);
    draw_path(f, settings, key_config, chunks[4]);
    draw_path_error(f, settings, chunks[5]);
    draw_help(f, settings, key_config, chunks[7]);
}

fn draw_volume(f: &mut Frame, settings: &SettingsState, key_config: &KeyConfig, area: Rect) {
    let selected = settings.selected() == SettingsField::Volume;
    let editing = selected && settings.is_editing_volume();

    let label = if editing {
        let left = key_hints::pick_binding_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsLeft,
            &[key_hints::kb(KeyCode::Left)],
        );
        let right = key_hints::pick_binding_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsRight,
            &[key_hints::kb(KeyCode::Right)],
        );
        let confirm = key_hints::pick_binding_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsConfirm,
            &[key_hints::kb(KeyCode::Enter)],
        );
        let cancel = key_hints::pick_binding_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsClose,
            &[key_hints::kb(KeyCode::Esc)],
        );

        format!(
            "Volume: {}%  [{}/{} adjust • 0-9 type • {} confirm • {} cancel]",
            settings.temp_volume(),
            key_hints::format_binding_opt(left),
            key_hints::format_binding_opt(right),
            key_hints::format_binding_opt(confirm),
            key_hints::format_binding_opt(cancel),
        )
    } else {
        format!("Volume: {}%", settings.temp_volume())
    };

    f.render_widget(Paragraph::new(label).style(field_style(selected)), area);
}

fn draw_repeat(f: &mut Frame, settings: &SettingsState, key_config: &KeyConfig, area: Rect) {
    let selected = settings.selected() == SettingsField::Repeat;
    let temp_repeat = settings.temp_repeat();

    let label = if selected {
        let left = key_hints::pick_binding_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsLeft,
            &[key_hints::kb(KeyCode::Left)],
        );
        let right = key_hints::pick_binding_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsRight,
            &[key_hints::kb(KeyCode::Right)],
        );
        let confirm = key_hints::pick_binding_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsConfirm,
            &[key_hints::kb(KeyCode::Enter)],
        );

        format!(
            "Repeat: {} {}  [{}/{} or {} to cycle]",
            temp_repeat.symbol(),
            repeat_label(temp_repeat),
            key_hints::format_binding_opt(left),
            key_hints::format_binding_opt(right),
            key_hints::format_binding_opt(confirm),
        )
    } else {
        format!(
            "Repeat: {} {}",
            temp_repeat.symbol(),
            repeat_label(temp_repeat),
        )
    };

    f.render_widget(Paragraph::new(label).style(field_style(selected)), area);
}

fn draw_prev_threshold(f: &mut Frame, settings: &SettingsState, key_config: &KeyConfig, area: Rect) {
    let selected = settings.selected() == SettingsField::PrevRestartThreshold;
    let editing = selected && settings.is_editing_prev_threshold();

    let label = if editing {
        let left = key_hints::pick_binding_with_preference(
            key_config, InputMode::Settings, InputAction::SettingsLeft,
            &[key_hints::kb(KeyCode::Left)],
        );
        let right = key_hints::pick_binding_with_preference(
            key_config, InputMode::Settings, InputAction::SettingsRight,
            &[key_hints::kb(KeyCode::Right)],
        );
        let confirm = key_hints::pick_binding_with_preference(
            key_config, InputMode::Settings, InputAction::SettingsConfirm,
            &[key_hints::kb(KeyCode::Enter)],
        );
        let cancel = key_hints::pick_binding_with_preference(
            key_config, InputMode::Settings, InputAction::SettingsClose,
            &[key_hints::kb(KeyCode::Esc)],
        );
        format!(
            "Prev restart: {}%  [{}/{} adjust • 0-9 type • {} confirm • {} cancel]",
            settings.temp_prev_threshold(),
            key_hints::format_binding_opt(left),
            key_hints::format_binding_opt(right),
            key_hints::format_binding_opt(confirm),
            key_hints::format_binding_opt(cancel),
        )
    } else if selected {
        let confirm = key_hints::pick_binding_with_preference(
            key_config, InputMode::Settings, InputAction::SettingsConfirm,
            &[key_hints::kb(KeyCode::Enter)],
        );
        format!(
            "Prev restart: {}%  [{} to edit]",
            settings.temp_prev_threshold(),
            key_hints::format_binding_opt(confirm),
        )
    } else {
        format!("Prev restart: {}%", settings.temp_prev_threshold())
    };

    f.render_widget(Paragraph::new(label).style(field_style(selected)), area);
}

fn draw_threshold_error(f: &mut Frame, settings: &SettingsState, area: Rect) {
    if let ThresholdValidation::Error(msg) = settings.threshold_validation() {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "  ✗ ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(msg.as_str(), Style::default().fg(Color::Red)),
            ])),
            area,
        );
    }
}


fn draw_path(f: &mut Frame, settings: &SettingsState, key_config: &KeyConfig, area: Rect) {
    let selected = settings.selected() == SettingsField::MusicPath;
    let confirm = key_hints::pick_binding_with_preference(
        key_config,
        InputMode::Settings,
        InputAction::SettingsConfirm,
        &[key_hints::kb(KeyCode::Enter)],
    );

    let label_color = if selected { Color::Yellow } else { Color::White };
    let hint_color = if selected { Color::Yellow } else { Color::DarkGray };

    let label: Line = if settings.is_editing_path() {
        Line::from(vec![
            Span::styled(
                "Music Path: ",
                Style::default()
                    .fg(label_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(settings.temp_path(), Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "  [Enter confirm • Esc cancel • Ctrl+U clear]",
                Style::default().fg(Color::Yellow),
            ),
        ])
    } else if settings.temp_path().is_empty() {
        let mut spans = vec![
            Span::styled("Music Path: ", Style::default().fg(label_color)),
            Span::styled("(not set)", Style::default().fg(Color::DarkGray)),
        ];
        if selected {
            spans.push(Span::styled(
                format!("  [{} to set]", key_hints::format_binding_opt(confirm)),
                Style::default().fg(hint_color),
            ));
        }
        Line::from(spans)
    } else {
        let mut spans = vec![
            Span::styled("Music Path: ", Style::default().fg(label_color)),
            Span::styled(settings.temp_path(), Style::default().fg(Color::Cyan)),
        ];
        if selected {
            spans.push(Span::styled(
                format!("  [{} to change]", key_hints::format_binding_opt(confirm)),
                Style::default().fg(hint_color),
            ));
        }
        Line::from(spans)
    };

    f.render_widget(Paragraph::new(label), area);
}

fn draw_path_error(f: &mut Frame, settings: &SettingsState, area: Rect) {
    if let PathValidation::Error(msg) = settings.path_validation() {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "  ✗ ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(msg.as_str(), Style::default().fg(Color::Red)),
            ])),
            area,
        );
    }
}

fn draw_help(f: &mut Frame, settings: &SettingsState, key_config: &KeyConfig, area: Rect) {
    let nav_up = key_hints::pick_binding_with_preference(
        key_config,
        InputMode::Settings,
        InputAction::SettingsNavigateUp,
        &[key_hints::kb(KeyCode::Up)],
    );
    let nav_down = key_hints::pick_binding_with_preference(
        key_config,
        InputMode::Settings,
        InputAction::SettingsNavigateDown,
        &[key_hints::kb(KeyCode::Down)],
    );
    let confirm = key_hints::pick_binding_with_preference(
        key_config,
        InputMode::Settings,
        InputAction::SettingsConfirm,
        &[key_hints::kb(KeyCode::Enter)],
    );
    let left = key_hints::pick_binding_with_preference(
        key_config,
        InputMode::Settings,
        InputAction::SettingsLeft,
        &[key_hints::kb(KeyCode::Left)],
    );
    let right = key_hints::pick_binding_with_preference(
        key_config,
        InputMode::Settings,
        InputAction::SettingsRight,
        &[key_hints::kb(KeyCode::Right)],
    );
    let close_keys = {
        let ordered = key_hints::ordered_bindings_with_preference(
            key_config,
            InputMode::Settings,
            InputAction::SettingsClose,
            &[key_hints::kb(KeyCode::Char('s')), key_hints::kb(KeyCode::Esc)],
        );
        let limited: Vec<_> = ordered.into_iter().take(2).collect();
        key_hints::format_bindings_join(&limited)
    };
    let cancel = key_hints::pick_binding_with_preference(
        key_config,
        InputMode::Settings,
        InputAction::SettingsClose,
        &[key_hints::kb(KeyCode::Esc)],
    );

    let text = if settings.is_editing_volume() {
        format!(
            "{}/{}: Adjust  •  0-9: Type value  •  {}: Confirm  •  {}: Cancel",
            key_hints::format_binding_opt(left),
            key_hints::format_binding_opt(right),
            key_hints::format_binding_opt(confirm),
            key_hints::format_binding_opt(cancel)
        )
    } else if settings.is_editing_prev_threshold() {
        format!(
            "{}/{}: Adjust  •  5-100: Type value  •  {}: Confirm  •  {}: Cancel",
            key_hints::format_binding_opt(left),
            key_hints::format_binding_opt(right),
            key_hints::format_binding_opt(confirm),
            key_hints::format_binding_opt(cancel),
        )
    } else if settings.is_editing_path() {
        "Type path  •  Enter: Confirm  •  Esc: Cancel  •  Ctrl+U: Clear".to_string()
    } else {
        match settings.selected() {
            SettingsField::Volume => format!(
                "{}/{}: Navigate  •  {}: Edit volume  •  {}: Close",
                key_hints::format_binding_opt(nav_up),
                key_hints::format_binding_opt(nav_down),
                key_hints::format_binding_opt(confirm),
                close_keys
            ),
            SettingsField::Repeat => {
                format!(
                    "{}/{}: Navigate  •  {}/{} or {}: Cycle mode  •  {}: Close",
                    key_hints::format_binding_opt(nav_up),
                    key_hints::format_binding_opt(nav_down),
                    key_hints::format_binding_opt(left),
                    key_hints::format_binding_opt(right),
                    key_hints::format_binding_opt(confirm),
                    close_keys
                )
            }
            SettingsField::PrevRestartThreshold => format!(
                "{}/{}: Navigate  •  {}: Edit threshold  •  {}: Close",
                key_hints::format_binding_opt(nav_up),
                key_hints::format_binding_opt(nav_down),
                key_hints::format_binding_opt(confirm),
                close_keys,
            ),
            SettingsField::MusicPath => format!(
                "{}/{}: Navigate  •  {}: Edit path  •  {}: Close",
                key_hints::format_binding_opt(nav_up),
                key_hints::format_binding_opt(nav_down),
                key_hints::format_binding_opt(confirm),
                close_keys
            ),
        }
    };

    f.render_widget(
        Paragraph::new(text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        area,
    );
}

fn field_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
