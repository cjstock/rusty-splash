use core::panic;

use anyhow::{anyhow, bail, Context, Ok};
use clap::{Parser, Subcommand};
use dialoguer::{Input, MultiSelect};
use display_info::DisplayInfo;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rusty_splash::{app::App, cache::Cached, cdragon::CDragon};

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
    Download { query: String },
}

#[derive(Debug, Subcommand)]
enum TileCommand {
    #[command()]
    Add { query: String },
    #[command()]
    Delete,
    #[command()]
    Build,
    #[command()]
    List,
    #[command()]
    New { name: Option<String> },
}

fn main() -> anyhow::Result<()> {
    let monitors = DisplayInfo::all()
        .with_context(|| "failed to get display info")?
        .iter()
        .map(|monitor| (monitor.width, monitor.height))
        .collect();
    let mut app = App::new(monitors)?;
    let mut cdragon = CDragon::new()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Tile(tile) => match tile {
            TileCommand::List => {
                let tile_names: Vec<String> =
                    app.tiles.iter().map(|tile| tile.name.clone()).collect();
                for name in tile_names {
                    println!("{name}");
                }
            }
            TileCommand::Build => todo!(),
            TileCommand::Delete => {
                let tiles = app.tiles.clone();
                let tile_names: Vec<String> = tiles.iter().map(|tile| tile.name.clone()).collect();
                if tile_names.is_empty() {
                    return Err(anyhow!("You haven't created any tiles yet!"));
                }
                let selected_tiles = MultiSelect::new()
                    .with_prompt("Which tiles do you want to delete?")
                    .report(false)
                    .items(&tile_names)
                    .interact_opt()
                    .unwrap();
                if let Some(selected_indicies) = selected_tiles {
                    for index in selected_indicies {
                        app.tile_delete(tiles[index].id)?;
                    }
                }
            }
            TileCommand::Add { query } => {
                let result_splashes = cdragon.query(query)?;
                let displayed_items: Vec<String> = result_splashes
                    .iter()
                    .map(|skin| skin.name.clone())
                    .collect();
                let selected_tile = app.selected_tile;
                let _ = MultiSelect::new()
                            .with_prompt("Select some skins using [space] and complete the selection with [enter]. Pressing [enter] without selecting any skins will use them all!")
                            .report(false)
                            .items(&displayed_items)
                            .interact_opt()
                            .unwrap()
                    .map(|selected| {
                        selected.iter().for_each(|selected_index| {
                            app.tile_add_splash(&selected_tile, &result_splashes[*selected_index].id).unwrap_or_else(|err| panic!("{err}"));

                        })
                    });
            }
            TileCommand::New { name } => {
                let name = name.unwrap_or_else(|| {
                    Input::new()
                        .with_prompt("What do you want to call it?")
                        .interact()
                        .unwrap()
                });
                let id = app.tile_new(name)?;
                app.tile_select(id)?;
            }
        },
        Commands::Download { query } => cdragon.query(query)?.into_par_iter().for_each(|skin| {
            CDragon::download_splash(skin, &app.download_path).unwrap_or_else(|e| panic!("{e}"))
        }),
    }

    Ok(())
}
