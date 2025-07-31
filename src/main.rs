use iced::Application;
use warp_terminal::WarpTerminal;
use tracing_subscriber::EnvFilter;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    WarpTerminal::run(iced::Settings::default())
}
