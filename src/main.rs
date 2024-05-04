use anyhow::{Context, Ok};
use clap::{Args, Parser, Subcommand};
use dialoguer::MultiSelect;
use display_info::DisplayInfo;
use rusty_splash::app::App;
use rusty_splash::cdragon::CDragon;
use rusty_splash::tiled_splash::build_tile;

#[derive(Parser, Debug)]
#[command(author = "Corey Stock", about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Tile(TileArgs),
    Preview { query: String },
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct TileArgs {
    #[command(subcommand)]
    command: Option<TileCommands>,
}

#[derive(Debug, Subcommand)]
enum TileCommands {
    Add { query: String },
    Remove,
    Build,
    List,
}

fn main() -> anyhow::Result<()> {
    let monitors = DisplayInfo::all()
        .with_context(|| "failed to get display info")?
        .iter()
        .map(|monitor| (monitor.width, monitor.height))
        .collect();
    let mut app = App::new(monitors);
    let mut data = CDragon::new()?;

    let query = "Blood Moon".to_string();

    let skins = data.query(query).map(|skins| {
        skins
            .iter()
            .map(|skin| skin.name.clone())
            .collect::<Vec<String>>()
    });
    dbg!(&skins);

    Ok(())
}
