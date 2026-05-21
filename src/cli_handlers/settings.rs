use crate::cli_handlers::context::CliContext;
use crate::cli_handlers::CliCommand;
use crate::core::events::{AppEvent, PlaybackEvent, UiEvent};
use crate::core::models::RepeatMode;
use crate::modules::ui::terminal::renderer::TerminalRenderer;
use crate::utils::{amplitude_to_volume, volume_percent_to_amplitude};
use anyhow::Result;

// ── Volume ────────────────────────────────────────────────────────────────────
pub struct VolumeCommand {
    pub volume: Option<u8>,
}

impl CliCommand for VolumeCommand {
    fn execute(self: Box<Self>) -> Result<()> {
        let ctx = CliContext::load()?;

        match self.volume {
            Some(vol) => {
                let volume_f32 = volume_percent_to_amplitude(vol);

                let mut app = CliContext::new_app(ctx)?;

                app.init()?;
                app.event_sender()
                    .send(AppEvent::Playback(PlaybackEvent::VolumeChanged { volume: volume_f32 }))?;
                app.run_once()?;
                app.cleanup()?;

                let ui = TerminalRenderer::new();
                ui.print_message(&format!("Volume set to: {}%", vol));
            }
            None => {
                let current_percent = amplitude_to_volume(ctx.state.config.volume);
                ctx.ui.print_message(&format!("Current volume: {}%", current_percent));
            }
        }

        Ok(())
    }
}

// ── Shuffle ───────────────────────────────────────────────────────────────────
pub struct ShuffleCommand {
    pub enabled: Option<bool>,
}

impl CliCommand for ShuffleCommand {
    fn execute(self :Box<Self>) -> Result<()> {
        let ctx = CliContext::load()?;

        let new_state = self.enabled.unwrap_or(!ctx.state.config.shuffle);
        ctx.ui.print_message(&format!("Shuffle set to: {}", new_state));

        let mut app = CliContext::new_app(ctx)?;

        app.init()?;
        app.event_sender()
            .send(AppEvent::Ui(UiEvent::ShuffleSet { enabled: new_state }))?;
        app.run_once()?;
        app.cleanup()?;
        Ok(())
    }
}

// ── Loop ─────────────────────────────────────────────────────────────
pub struct LoopCommand {
    pub mode: Option<RepeatMode>,
}

impl CliCommand for LoopCommand {
    fn execute(self: Box<Self>) -> Result<()> {
        let ctx = CliContext::load()?;

        let new_mode = self.mode.unwrap_or_else(|| ctx.state.config.repeat.cycle());
        ctx.ui.print_message(&format!(
            "Repeat mode set to: {} {}",
            new_mode.symbol(),
            repeat_mode_description(new_mode),
        ));

        let mut app = CliContext::new_app(ctx)?;

        app.init()?;
        app.event_sender()
            .send(AppEvent::Playback(PlaybackEvent::RepeatChanged { mode: new_mode }))?;
        app.run_once()?;
        app.cleanup()?;

        Ok(())
    }
}

/// Human-readable label used in terminal feedback messages
fn repeat_mode_description(mode: RepeatMode) -> &'static str {
    match mode {
        RepeatMode::Off => "(stop at end)",
        RepeatMode::All => "(loop playlist)",
        RepeatMode::One => "(repeat current song)",
    }
}

// ── PrevThreshold ────────────────────────────────────────────────────
pub struct PrevThresholdCommand {
    pub threshold: Option<u8>,
}

impl CliCommand for PrevThresholdCommand {
    fn execute(self: Box<Self>) -> Result<()> {
        let ctx = CliContext::load()?;

        match self.threshold {
            Some(threshold) => {
                let mut app = CliContext::new_app(ctx)?;

                app.init()?;
                app.event_sender()
                    .send(AppEvent::Ui(UiEvent::PrevThresholdSet { threshold }))?;
                app.run_once()?;
                app.cleanup()?;


                let ui = TerminalRenderer::new();
                ui.print_message(&format!("Restart threshold: {}%", threshold));
            }

            None => {
                ctx.ui.print_message(&format!("Restart threshold: {}%", ctx.state.config.prev_restart_threshold));
            }
        }

        Ok(())
    }
}