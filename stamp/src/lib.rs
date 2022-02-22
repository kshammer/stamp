mod stamp;
use eframe::{
    egui::{
        CentralPanel,
    },
    epi::App,
};
use tracing::{event, Level};
pub use stamp::{Stamp, Player};

impl App for Stamp{

    fn setup(&mut self, _ctx: &eframe::egui::CtxRef, _frame: &eframe::epi::Frame, _storage: Option<&dyn eframe::epi::Storage>){
        self.fetch_players("83615933");
        self.fetch_players("130643254");
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, _frame: &eframe::epi::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Dota");
            for player in self.players.iter(){
                player.render_player(ui);
            }
        });
    }


    fn name(&self) -> &str {
        "Dota Stamp"
    }

}

