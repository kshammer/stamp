use stampcore::OpenDotaAPI;
use tracing::{event, Level};

pub struct Player {
    pub player_name : String,
    pub player_pic_url: String,
    pub player_account_id: String,
}

impl Player {
    pub fn new() -> Player{
        Player {
            player_name: "lol".to_string(),
            player_pic_url: "pending".to_string(),
            player_account_id: "1234".to_string()
        }
    }
    pub fn render_player(&self, ui:&mut eframe::egui::Ui){
        let name = format!("Name: {}", self.player_name);
        let account_id = format!("Account_ID: {}", self.player_account_id);
        ui.label(name);
        ui.label(account_id);
    }
}

pub struct Stamp{
    pub players: Vec<Player>
}

impl Stamp {
    pub fn new() -> Stamp{
        Stamp{
            players: vec![]
        }
    }

    pub fn fetch_players(&mut self, player_id: &str){
        event!(Level::INFO, "something has happened!");
        if let Ok(response) = OpenDotaAPI::fetch_player(player_id){
            let player = Player{
                player_name: response.username,
                player_account_id: response.account_id,
                player_pic_url: response.profile_picture
            };
            self.players.push(player)
        }
    }
}