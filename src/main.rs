use clap::{Parser, Subcommand};
use rust_splash::champs::Splashes;

#[derive(Parser, Debug)]
#[command(author = "Corey")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Champion { name: Option<String> },
    SkinLine { name: Option<String> },
}

fn main() {
    let data = Splashes::new();
    data.save_data();
    let args = Cli::parse();

    match &args.command {
        Commands::Champion { name } => {
            if let Some(name) = name {
                println!("{:?}", data.splashes_for_champ(name))
            }
        }
        Commands::SkinLine { name } => {
            if let Some(skin_line) = name {
                println!("{:?}", data.skin_line(skin_line))
            }
        }
    }
}
