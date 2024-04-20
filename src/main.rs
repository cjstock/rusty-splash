use clap::{Args, Parser, Subcommand};
use dialoguer::MultiSelect;
use rusty_splash::app::App;
use rusty_splash::cdragon::CDragon;
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
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    window.set_visible(false);
    let monitors = monitors(&window);
    let mut app = App::new(monitors);
    let mut data = CDragon::new();

    let skin = data
        .skin(117000)
        .unwrap_or_else(|| panic!("that's not a fucking skin!"));

    if let Ok(_) = CDragon::download_splash(skin, &app.download_path) {
        app.add_download(skin.id);
    }
}
