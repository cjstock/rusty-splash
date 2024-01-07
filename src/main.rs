use std::process::exit;

use clap::{Parser, Subcommand};
use rust_splash::champs::{self, Splashes};

#[derive(Parser, Debug)]
#[command(author = "Corey Stock", about = "Explore LoL skins and splash arts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ///Lists the skins for a champion
    Champion { query: Option<String> },
    ///Search for skins by name
    Splashes {
        ///Preview skins in browser
        #[arg(short, long, default_value_t = false)]
        preview: bool,
        query: Option<String>,
    },
}

fn main() {
    let data = Splashes::new();
    data.save_data();
    let args = Cli::parse();

    match &args.command {
        Commands::Champion { query } => {
            if let Some(name) = query {
                for skin in data.splashes_for_champ(name) {
                    println!("{:?}", skin);
                }
            }
        }
        Commands::Splashes {
            query: name,
            preview,
        } => {
            if let Some(query) = name {
                let skins = data.skin_line(query);
                if *preview {
                    match skins.len() {
                        0 => {
                            println!("No results found! Try again...");
                        }
                        1 => {
                            let (name, skin_data) = data.skin(&skins[0].name).unwrap();
                            let _ = champs::preview(&name, skin_data.num);
                            exit(0);
                        }
                        _ => {
                            println!("Select skins from the list to preview using their id's separated by spaces (Ex: 1 2 4):");
                            for (index, skin) in skins.iter().enumerate() {
                                println!("  {index}: {:?}", skin.name);
                            }

                            let mut input = String::default();
                            let _ = std::io::stdin().read_line(&mut input);

                            match input.trim() {
                                "" => {
                                    for skin in skins.iter() {
                                        if let Some((champ, skin_data)) = data.skin(&skin.name) {
                                            let _ = champs::preview(&champ, skin_data.num);
                                        }
                                    }
                                }
                                _ => {
                                    let selected_skins =
                                        input.trim().split(' ').map(|val| val.parse::<usize>());
                                    for selected_skin in selected_skins.flatten() {
                                        let selected_name = &skins[selected_skin].name;
                                        if let Some((champ, skin_data)) = data.skin(selected_name) {
                                            let _ = champs::preview(&champ, skin_data.num);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for (index, skin) in skins.iter().enumerate() {
                        println!("  {index}: {:?}", skin.name);
                    }
                }
            }
        }
    }
}
