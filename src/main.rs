use clap::{Parser, Subcommand};
use image::GenericImageView;
use rusty_splash::datadragon::preview_splash;
use rusty_splash::splashes::Splashes;
use rusty_splash::tiled_splash::{aspect_ratio, how_many_fit, monitors};
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

#[derive(Debug)]
struct TileParams {
    dims: (u32, u32),
    image_res: (u32, u32),
    image_adjust: (u32, u32),
}

fn calculate_tile_y_bias(image_res: (u32, u32), container_res: (u32, u32), c: u32) -> TileParams {
    let image_res = (image_res.0 as f32, image_res.1 as f32);
    let container_res = (container_res.0 as f32, container_res.1 as f32);
    let fit_y = container_res.1 / image_res.1;
    let target_fit_y = fit_y.ceil() + (c as f32);
    let new_image_y = container_res.1 / target_fit_y;
    let image_ar = image_res.0 / image_res.1;
    let new_image_x = image_ar * new_image_y;
    let fit_x = container_res.0 / new_image_x;
    let overfit = fit_x.ceil();
    let overfit_error = fit_x.ceil() - fit_x;
    let pixel_x_error = overfit_error * new_image_x;
    let x_adjust = pixel_x_error / overfit;
    TileParams {
        dims: (overfit as u32, target_fit_y as u32),
        image_res: (new_image_x.round() as u32, new_image_y.round() as u32),
        image_adjust: (x_adjust.round() as u32, 0),
    }
}
fn calculate_tile_x_bias(image_res: (u32, u32), container_res: (u32, u32), c: u32) -> TileParams {
    let image_res = (image_res.0 as f32, image_res.1 as f32);
    let container_res = (container_res.0 as f32, container_res.1 as f32);
    let fit_x = container_res.0 / image_res.0;
    let target_fit_x = fit_x.ceil() + (c as f32);
    let new_image_x = container_res.0 / target_fit_x;
    let image_ar = image_res.0 / image_res.1;
    let new_image_y = new_image_x / image_ar;
    let fit_y = container_res.1 / new_image_y;
    let overfit = fit_y.ceil();
    let overfit_error = fit_y.ceil() - fit_y;
    let pixel_y_error = overfit_error * new_image_y;
    let y_adjust = pixel_y_error / overfit;
    TileParams {
        dims: (target_fit_x.round() as u32, overfit.round() as u32),
        image_res: (new_image_x.round() as u32, new_image_y.round() as u32),
        image_adjust: (0, y_adjust.round() as u32),
    }
}

fn main() {
    let data = Splashes::new();
    let args = Cli::parse();
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();

    let monitors = monitors(&window);
    let mut draven_path = data.save_dir.clone();
    draven_path.push("Draven_0.jpg");
    let image = image::open(draven_path.clone()).unwrap();

    let y = calculate_tile_y_bias(
        image.dimensions(),
        (monitors[0].width, monitors[0].height),
        0,
    );
    let x = calculate_tile_x_bias(
        image.dimensions(),
        (monitors[0].width, monitors[0].height),
        0,
    );

    println!("x: {:?}", x);
    println!("y: {:?}", y);

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
