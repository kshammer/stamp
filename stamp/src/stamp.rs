pub struct Stamp {
    pub player_data : String
}

impl Stamp {
    pub fn  new() -> Stamp{
        Stamp {
            player_data: "lol".to_string()
        }
    }
    pub fn render_player(&self, ui:&mut eframe::egui::Ui){
        let player = format!("Cool {}", self.player_data);
        ui.label(player);
    }
}