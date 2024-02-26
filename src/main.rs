use clap::{Args, Parser, Subcommand};
use dialoguer::MultiSelect;
use rusty_splash::datadragon::preview_splash;
use rusty_splash::splashes::Splashes;
use rusty_splash::tiled_splash::{build_tile, monitors};
use winit::event_loop::EventLoop;
use winit::window::Window;

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

fn main() {
    let mut data = Splashes::new();
    let args = Cli::parse();

    match &args.command {
        Commands::Tile(tile) => {
            let tile_comand = tile.command.as_ref().unwrap_or(&TileCommands::Build);
            match tile_comand {
                TileCommands::Build => {
                    let event_loop = EventLoop::new().unwrap();
                    let window = Window::new(&event_loop).unwrap();
                    window.set_visible(false);
                    let monitors = monitors(&window);
                    let mut paths =
                        data.download_ids(data.app_state.tile_imgs.clone().into_iter().collect());
                    build_tile(&mut paths, monitors[0].into());
                }
                TileCommands::Add { query } => {
                    let splashes = data.search_skins(query);
                    let options: Vec<String> =
                        splashes.iter().map(|splash| splash.name.clone()).collect();
                    let selection = MultiSelect::new()
                        .with_prompt("Select all with 'a'")
                        .report(false)
                        .items(&options)
                        .interact()
                        .unwrap()
                        .iter()
                        .map(|i| splashes[*i].id.to_string())
                        .collect();
                    data.add_tiled_ids(selection);
                }
                TileCommands::Remove => {
                    let splashes = data
                        .get_skins_by_ids(&data.app_state.tile_imgs.clone().into_iter().collect());
                    if splashes.is_empty() {
                        return;
                    }
                    let options: Vec<String> =
                        splashes.iter().map(|splash| splash.name.clone()).collect();
                    let selection = MultiSelect::new()
                        .with_prompt("Select all with 'a'")
                        .report(false)
                        .items(&options)
                        .interact()
                        .unwrap()
                        .iter()
                        .map(|i| splashes[*i].id.to_string())
                        .collect();
                    data.remove_tiled_ids(selection);
                }
                TileCommands::List => {
                    let splashes = data
                        .get_skins_by_ids(&data.app_state.tile_imgs.clone().into_iter().collect());
                    splashes.iter().for_each(|skin| println!("{:?}", skin.name))
                }
            }
        }
        Commands::Preview { query } => {
            let splashes = data.search_skins(query);
            let options: Vec<String> = splashes.iter().map(|splash| splash.name.clone()).collect();
            let selection: Vec<&rusty_splash::splashes::Skin> = MultiSelect::new()
                .with_prompt("Select all with 'a'")
                .report(false)
                .items(&options)
                .interact()
                .unwrap()
                .iter()
                .map(|i| splashes[*i])
                .collect();
            selection
                .iter()
                .for_each(|skin| preview_splash(skin).unwrap());
        }
    }
}
