mod browse;
mod list;
mod path;
mod play;
mod playlist;
mod refresh;
mod search;
mod select;
mod settings;
mod sort;
mod status;
mod context;

pub use browse::BrowseCommand;
pub use list::ListCommand;
pub use path::PathCommand;
pub use play::PlayCommand;
pub use playlist::PlaylistCommand;
pub use refresh::RefreshCommand;
pub use search::SearchCommand;
pub use select::SelectCommand;
pub use settings::{LoopCommand, ShuffleCommand, VolumeCommand, PrevThresholdCommand};
pub use sort::SortCommand;
pub use status::StatusCommand;

use crate::cli::Commands;
use anyhow::Result;

/// Every CLI command implements this trait.
///
/// Commands own their arguments and are consumed on execution — they run exactly once.
pub trait CliCommand {
    fn execute(self: Box<Self>) -> Result<()>;
}

/// Converts a parsed [`Commands`] variant into a boxed [`CliCommand`] ready to execute.
///
/// Keeping this in one place means `main.rs` never needs to know about concrete command types.
pub fn from_cli(cmd: Commands) -> Box<dyn CliCommand> {
    match cmd {
        Commands::Browse => Box::new(BrowseCommand),
        Commands::Play { file } => Box::new(PlayCommand { file }),
        Commands::Path { directory } => Box::new(PathCommand { directory }),
        Commands::Refresh => Box::new(RefreshCommand),
        Commands::Playlist => Box::new(PlaylistCommand),
        Commands::List => Box::new(ListCommand),
        Commands::Select { index } => Box::new(SelectCommand { index }),
        Commands::Search { query } => Box::new(SearchCommand { query }),
        Commands::Volume { volume } => Box::new(VolumeCommand { volume }),
        Commands::Shuffle { enabled } => Box::new(ShuffleCommand { enabled }),
        Commands::Loop { mode } => Box::new(LoopCommand { mode }),
        Commands::Sort { by } => Box::new(SortCommand { field: by }),
        Commands::Status => Box::new(StatusCommand),
        Commands::PrevThreshold { threshold } => Box::new(PrevThresholdCommand { threshold }),
    }
}