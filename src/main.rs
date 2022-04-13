#![feature(windows_by_handle)]
use iced::{
    image, Application, Clipboard, Column, Command, Container, Element, Image, Length, Settings, Text, Row
};
use log_watch::LogWatcher;
use opendota_client::apis::{
    configuration, players_api::players_account_id_get, players_api::players_account_id_wl_get,
};
use opendota_client::models::PlayerResponseProfile;
use opendota_client::models::{player_response::PlayerResponse, PlayerWinLossResponse};
use reqwest;
use std::future::Future;
use tracing::{info};
use tracing_subscriber;

mod log_watch;
pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_env_filter("stamp=trace")
        .init();
    info!("Starting program");
    Stamp::run(Settings::default())
}

#[derive(Default)]
struct Stamp {
    dota_match: DotaMatch,
}
#[derive(Debug)]
enum Message {
    LookingForMatch,
    MatchFound(Vec<i32>),
    PlayerLoaded(DotaPlayer),
}

impl Application for Stamp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self::default(), 
            Command::perform(watch(), |player_ids| Message::MatchFound(player_ids)),
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
            Message::LookingForMatch => {Command::none()}
            Message::MatchFound(player_ids) => {
                info!("Found Players");
                let commands = player_ids
                    .iter()
                    .map(|player_id| {
                        Command::perform(DotaPlayer::fetch_player_info(*player_id), |player_card| {
                            Message::PlayerLoaded(player_card)
                        })
                    })
                    .collect::<Vec<_>>();
                Command::batch(commands)
            }
            Message::PlayerLoaded(player) => {
                info!("Loaded Player");
                self.dota_match.players.push(player);
                Command::none()
            }
        }

    }

    fn view(&mut self) -> Element<Self::Message> {

        let content = if self.dota_match.players.is_empty() {
            let row = Row::new();
            row.push(Text::new("Looking for matches"))
            
        } else {
            let mut elements = Vec::<Element<Message>>::new();
            for player in self.dota_match.players.clone(){
                elements.push(DotaPlayer::view(player));
            }
            Row::with_children(elements).into()
        };
        

        Container::new(content).width(Length::Fill).height(Length::Fill).into()
    }
}

fn watch() -> impl Future<Output = Vec<i32>> {
    async {
        let mut log_watcher = LogWatcher::register("log.txt").unwrap();
        let player_ids = log_watcher.watch().await;
        // let player_ids = vec![83615933, 207041414, 218061707, 346964866];
        player_ids
    }
}

#[derive(Clone)]
struct DotaMatch {
    players: Vec<DotaPlayer>,
}

impl Default for DotaMatch {
    fn default() -> Self {
        Self::new()
    }
}

impl DotaMatch {
    pub fn new() -> Self {
        Self {
            players: vec![],
        }
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
    async fn fetch_player_image(url: String) -> Result<image::Handle, reqwest::Error> {
        let bytes = reqwest::get(&url).await?.bytes().await?;
        Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
    }


    pub fn fetch_player_info(id: i32) -> impl Future<Output = DotaPlayer> {
        info!("Fetching Player");
        async move{
            info!("In async");
            let mut player = DotaPlayer::new();
            let response =
                match players_account_id_get(&configuration::Configuration::default(),id).await {
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
            let profile = match response.profile {
                Some(x) => x,
                None => {
                    let private_profile = PlayerResponseProfile::new();
                    Box::new(private_profile)
                }
            };
            let profile_name = *profile;
            player.name = match profile_name.personaname {
                Some(x) => x,
                None => "Private Profile".to_string(),
            };
            player.image_url = match profile_name.avatarfull {
                Some(x) => x,
                None => "".to_string(),
            };

            player.image = match Self::fetch_player_image(player.image_url.clone()).await {
                Ok(x) => x,
                Err(_) => image::Handle::from_path("resources/default.png"),
            };

            let response = match players_account_id_wl_get(
                &configuration::Configuration::default(),
                id,
                Some(20),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            {
                Ok(x) => x,
                Err(_) => PlayerWinLossResponse {
                    win: None,
                    lose: None,
                },
            };
            player.wins = match response.win {
                Some(x) => x.to_string(),
                None => "".to_string(),
            };
            player.losses = match response.lose {
                Some(x) => x.to_string(),
                None => "".to_string(),
            };
            info!("finished");
            player
        }
    }

    pub fn new() -> Self {
        Self {
            name: String::from("Player"),
            image: image::Handle::from_path("resources/default.png"), // replace with empty image?
            image_url: String::from("None"),
            wins: String::from("-1"),
            losses: String::from("-1"),
        }
    }

    pub fn view(self) -> Element<'static, Message> {
        Column::new()
            .push(Text::new(self.name))
            .push(
                Image::new(self.image)
                    .width(Length::Units(100))
                    .height(Length::Units(100)),
            )
            .push(Text::new(format!("Wins {}", self.wins)))
            .push(Text::new(format!("Loses {}", self.losses)))
            .into()
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
}
