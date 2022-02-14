use eframe::{run_native, NativeOptions};
use stamp::Stamp;

fn main() {
    tracing_subscriber::fmt::init();

    let app = Stamp::new();
    let win_option = NativeOptions::default();
    run_native(Box::new(app), win_option);
}