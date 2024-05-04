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
    #[command(subcommand)]
    Tile(TileCommand),
    #[command()]
    Preview { query: String },
}

#[derive(Debug, Subcommand)]
enum TileCommand {
    #[command()]
    Add { query: String },
    #[command()]
    Remove,
    #[command()]
    Build,
    #[command()]
    List,
}

fn main() -> anyhow::Result<()> {
    let monitors = DisplayInfo::all()
        .with_context(|| "failed to get display info")?
        .iter()
        .map(|monitor| (monitor.width, monitor.height))
        .collect();
    let mut app = App::new(monitors);
    let mut cdragon = CDragon::new()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Tile(tile) => match tile {
            TileCommand::List => todo!(),
            TileCommand::Build => todo!(),
            TileCommand::Remove => todo!(),
            TileCommand::Add { query } => {
                let result_splashes = cdragon.query(query);
                let displayed_items: Option<Vec<String>> = result_splashes.map_or(None, |skins| {
                    Some(
                        skins
                            .into_iter()
                            .map(|skin| skin.name.to_string())
                            .collect(),
                    )
                });
                match displayed_items {
                    Some(skins) => {
                        let selected_skins = MultiSelect::new()
                            .with_prompt("Select some skins using [space] and complete the selection with [enter]. Pressing [enter] without selecting any skins will use them all!")
                            .report(false)
                            .items(&skins)
                            .interact_opt()
                            .unwrap();
                    }
                    None => println!("No skins found for that query!"),
                }
            }
        },
        Commands::Preview { query } => todo!(),
    }

    Ok(())
}
