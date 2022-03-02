use stampcore::OpenDotaAPI;
use tracing::{event, Level};

pub struct Player {
    pub player_name : String,
    pub player_pic_url: String,
    pub player_account_id: String,
    pub win: i64,
    pub lose: i64
}

impl Player {
    pub fn new() -> Player{
        Player {
            player_name: "lol".to_string(),
            player_pic_url: "pending".to_string(),
            player_account_id: "1234".to_string(),
            win: -1,
            lose: -1
        }
    }
    pub fn render_player(&self, ui:&mut eframe::egui::Ui){
        let name = format!("Name: {}", self.player_name);
        let account_id = format!("Account_ID: {}", self.player_account_id);
        ui.label(name);
        ui.label(account_id);
        let win = format!("Win: {}", self.win);
        ui.label(win);
        let lose = format!("Lose: {}", self.lose);
        ui.label(lose);
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
        let mut player = Player::new();
        let response = OpenDotaAPI::fetch_player(player_id).unwrap();
        player.player_name = response.username;
        player.player_account_id = response.account_id;
        player.player_pic_url = response.profile_picture;
        let winlose = OpenDotaAPI::fetch_player_wl(player_id).unwrap();
        player.win = winlose.win;
        player.lose = winlose.lose;
        
        self.players.push(player)
        
    }
}