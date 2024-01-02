use clap::Parser;
use rust_splash::champs::ChampionSkins;

#[derive(Parser, Debug)]
#[command(author = "Corey")]
struct Cli {
    #[arg(short, long)]
    champion: String,
    #[arg(short, long, default_value_t = 0)]
    skin_number: usize,
}

fn main() {
    let data = ChampionSkins::load();
    let args = Cli::parse();

    let selected = data.skins_for(&args.champion);
    println!("{:?}", selected)
}
