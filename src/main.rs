mod app;
mod charts;
mod model;

use app::PokerTrainerApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([980.0, 680.0])
            .with_title("Poker Trainer")
            .with_decorations(false),
        ..Default::default()
    };

    eframe::run_native(
        "Poker Trainer",
        native_options,
        Box::new(|cc| Ok(Box::new(PokerTrainerApp::new(cc)))),
    )
}
