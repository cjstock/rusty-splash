use rust_splash::champs::ChampionSkins;

#[tokio::main]
async fn main() {
    let everything = ChampionSkins::new().await;
    println!("{:?}", everything)
}
