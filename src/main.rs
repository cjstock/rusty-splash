use clap::{Parser, Subcommand};
use rusty_splash::datadragon::preview_splash;
use rusty_splash::splashes::Splashes;
use rusty_splash::tiled_splash::monitors;
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
    ///Lists the splashes for a champion
    Champion { query: Option<String> },
    ///Preview or Download champion splash art.
    Get {
        ///Preview splashes in browser - be carful or you might open a lot of browser tabs >:^P
        #[arg(short, long, default_value_t = false)]
        preview: bool,
        ///Download splashes to $HOME/rusty-splash/splashes
        #[arg(short, long, default_value_t = false)]
        download: bool,
        query: String,
    },
}

fn main() {
    let data = Splashes::new();
    let args = Cli::parse();
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();

    let dims = monitors(&window);

    println!("dimensions: {:?}", dims);

    match &args.command {
        Commands::Champion { query } => {
            if let Some(name) = query {
                for skin in data.splashes_for_champ(name) {
                    println!("{:?}", skin);
                }
            }
        }
        Commands::Get {
            query,
            preview,
            download,
        } => {
            let skins = data.search_skins(query);
            if *preview || *download {
                match skins.len() {
                    0 => {
                        println!("No results found! Try again...");
                    }
                    1 => {
                        let skin_data = data.skin(&skins[0].name).unwrap();
                        if *preview {
                            let _ = preview_splash(skin_data);
                        }
                        if *download {
                            let _ = data.download(skin_data);
                        }
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
                                    if let Some(skin_data) = data.skin(&skin.name) {
                                        if *preview {
                                            let _ = preview_splash(skin_data);
                                        }
                                        if *download {
                                            let _ = data.download(skin_data);
                                        }
                                    }
                                }
                            }
                            _ => {
                                let selected_skins =
                                    input.trim().split(' ').map(|val| val.parse::<usize>());
                                for selected_skin in selected_skins.flatten() {
                                    let selected_name = &skins[selected_skin].name;
                                    if let Some(skin_data) = data.skin(selected_name) {
                                        if *preview {
                                            let _ = preview_splash(skin_data);
                                        }
                                        if *download {
                                            let _ = data.download(skin_data);
                                        }
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
