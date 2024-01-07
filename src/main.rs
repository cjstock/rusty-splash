use clap::Parser;
use rust_splash::champs::Splashes;

#[derive(Parser, Debug)]
#[command(author = "Corey")]
struct Cli {
    #[arg(short, long)]
    champion: String,
    #[arg(short, long, default_value_t = 0)]
    skin_number: usize,
}

fn main() {
    let data = Splashes::new();
    data.save_data();
    let args = Cli::parse();

    let selected = data.splashes_for_champ(&args.champion);
    println!("{:?}", selected)
}
