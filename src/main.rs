#![feature(windows_by_handle)]
use iced::{
    image, Application, Clipboard, Column, Command, Container, Element, Image, Length, Settings,
    Text, Row,
};
use opendota_client::apis::{configuration, players_api::players_account_id_get, players_api::players_account_id_wl_get};
use opendota_client::models::{player_response::PlayerResponse, PlayerWinLossResponse};
use reqwest;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use regex::Regex;
use lazy_static::lazy_static;
use itertools::Itertools;
mod log_watch;
use log_watch::{LogWatcher, LogWatcherAction};

pub fn main() -> iced::Result {
    Stamp::run(Settings::default())
}

#[derive(Debug)]
enum Stamp {
    Loading,
    Loaded { dota_match: DotaMatch },
}

#[derive(Debug, Clone)]
enum Message {
    DotaMatchFound(Result<DotaMatch, Error>),
}

impl Application for Stamp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Stamp, Command<Self::Message>) {
        (
            Stamp::Loading,
            Command::perform(watch(), Message::DotaMatchFound),
        )
    }

    fn title(&self) -> String {
        String::from("Dota 2 Stamp")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::DotaMatchFound(Ok(dota_match)) => {
                *self = Stamp::Loaded { dota_match: dota_match };
                Command::none()
            }
            Message::DotaMatchFound(Err(_error)) => Command::none(),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = match self {
            Stamp::Loading => Column::new().push(Text::new("Searching for Dota Players")),
            Stamp::Loaded { dota_match } => Column::new().push(dota_match.view()),
        };
        Container::new(content).into()
    }
}

async fn watch() -> Result<DotaMatch, Error>{
    // let (sender, receiver): (Sender<String>, Receiver<String>) = channel();
    // let t = thread::spawn(move || {
    //     let mut log_watcher = LogWatcher::register("log.txt").unwrap(); 
    //     log_watcher.watch(&sender);
    // });
    // let log_string = receiver.recv().unwrap();
    // t.join().unwrap();
    // print!("{}", log_string);
    // if dota_match_log_message(&log_string) {
    //     let ids = fetch_player_ids(&log_string);
    //     let dota_match = DotaMatch::create_players(ids).await;
    //     return Ok(dota_match);
    // }
    let dota_match = DotaMatch::create_players(vec![83615933,68167571]).await;
    Ok(dota_match)
}



fn fetch_player_ids(text: &str) -> Vec<i32> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[U:1:\d{8,9}\]").unwrap(); // rust can't use positive or negative lookaheads 
    }
    // matches [U:1:370898177]
    let full_ids: Vec<String> = RE.find_iter(text)
        .filter_map(|matches| matches.as_str().parse().ok())
        .collect();

    // trims the [U:1:]
    let string_ids = full_ids.iter().unique().map(|x| &x[5..x.len()-1]).map(|x| x.to_string()).collect::<Vec<_>>();

    string_ids.iter().map(|id| id.parse::<i32>().unwrap()).collect()
} 

fn dota_match_log_message(text: &str) -> bool {
    let re = Regex::new(r"Lobby").unwrap();
    re.is_match(text)
}

#[derive(Debug, Clone)]
struct DotaMatch {
    players: Vec<DotaPlayer>
}

impl DotaMatch {
    pub fn new() -> Self {
        Self {
            players: vec![]
        }
    }

    async fn create_players(ids: Vec<i32>) -> DotaMatch{
        let mut dota_match = DotaMatch::new();
        for id in ids.iter() {
            dota_match.players.push(DotaPlayer::fetch_player_info(*id).await)
        }
        dota_match
    }

    fn view(&mut self) -> Element<Message>{
        let mut elements = Vec::<Element<Message>>::new();
        for player in self.players.clone(){
            elements.push(DotaPlayer::view(player));
        }
        Row::with_children(elements).into()
    }
}


#[derive(Debug, Clone)]
struct DotaPlayer {
    name: String,
    image: image::Handle,
    image_url: String,
    wins: String,
    losses: String,
}

impl DotaPlayer {
    pub fn new() -> Self {
        Self {
            name: String::from("Player"),
            image: image::Handle::from_path("resources/default.png"), // replace with empty image? 
            image_url: String::from("None"),
            wins: String::from("-1"),
            losses: String::from("-1"),
        }
    }

    pub async fn fetch_player_info(id: i32) -> DotaPlayer{
        let mut player = DotaPlayer::new();
        let response = match players_account_id_get(
            &configuration::Configuration::default(),
            id,
        )
        .await
        {
            Ok(x) => x,
            Err(_) => PlayerResponse {
                tracked_until: None,
                solo_competitive_rank: None,
                competitive_rank: None,
                rank_tier: None,
                leaderboard_rank: None,
                mmr_estimate: None,
                profile: None,
            },
        };
        let profile = response.profile.unwrap(); // this will panic on private accounts 
        let profile_name = *profile;
        player.name = match profile_name.personaname {
            Some(x) => x,
            None => "No name".to_string(),
        };
        player.image_url = match profile_name.avatarfull {
            Some(x) => x,
            None => "No url".to_string(),
        };

        player.image = match Self::fetch_player_image(player.image_url.clone()).await {
            Ok(x) => x,
            Err(_) => image::Handle::from_path("resources/default.png"),
        };

        let response =  match players_account_id_wl_get(
            &configuration::Configuration::default(),
            id,
            Some(20),None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None
        )
        .await
        {
            Ok(x) => x,
            Err(_) => PlayerWinLossResponse {
                win: None,
                lose: None
            },
        };
        player.wins = match response.win{
            Some(x) => x.to_string(),
            None => "-1".to_string()
        };
        player.losses = match response.lose{
            Some(x) => x.to_string(),
            None => "-1".to_string()
        };

        player
    }

    fn view(player: DotaPlayer) -> Element<'static, Message> {
        Column::new()
            .push(Text::new(&player.name))
            .push(
                Image::new(player.image.clone())
                    .width(Length::Units(100))
                    .height(Length::Units(100)),
            )
            .push(Text::new(format!("Wins {}", &player.wins)))
            .push(Text::new(format!("Loses {}", &player.losses))).into()
    }

    async fn fetch_player_image(url: String) -> Result<image::Handle, reqwest::Error> {
        let bytes = reqwest::get(&url).await?.bytes().await?;
        Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
}
