use hexcaster::game::Game;

fn main() {
    if let Err(e) = Game::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
