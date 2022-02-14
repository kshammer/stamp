mod stamp;
use eframe::{
    egui::{
        CentralPanel,
    },
    epi::App,
};
use stampcore::OpenDotaAPI;
use tracing::{event, Level};
pub use stamp::{Stamp};

impl App for Stamp{

    fn setup(&mut self, ctx: &eframe::egui::CtxRef, _frame: &eframe::epi::Frame, _storage: Option<&dyn eframe::epi::Storage>){
        fetch_player(&mut self.player_data)
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, _frame: &eframe::epi::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Dota");
            self.render_player(ui);
        });
    }


    fn name(&self) -> &str {
        "Stamp"
    }
}

fn fetch_player(player_data: &mut String){
    event!(Level::INFO, "something has happened!");
    if let Ok(response) = OpenDotaAPI::new().fetch(){
        print!("{}", response.to_string());
        *player_data = response;
    }
}