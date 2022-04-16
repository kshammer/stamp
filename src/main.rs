#![feature(windows_by_handle)]
use iced::{
    image, Application, Clipboard, Column, Command, Container, Element, Image, Length, Row,
    Settings, Text,
};
use log_watch::LogWatcher;
use opendota_client::apis::{
    configuration, players_api::players_account_id_get, players_api::players_account_id_wl_get,
};
use opendota_client::models::PlayerResponseProfile;
use opendota_client::models::{player_response::PlayerResponse, PlayerWinLossResponse};
use reqwest;
use std::future::Future;
use tracing::info;
use tracing_subscriber;
mod style;

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
            Message::LookingForMatch => Command::none(),
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
        let row_spacing = 10; 
        let col_spacing = 10;
        let (top_row, bot_row) = match self.dota_match.players.len() {
            0 => {
                let top_row = Row::new().push(Text::new("Looking for matches"));
                (top_row, Row::new())
            }
            1..=4 => {
                let top_row = self
                    .dota_match
                    .players
                    .clone()
                    .iter()
                    .fold(Row::new(), |row, player| row.push(player.view())).spacing(row_spacing);
                let bot_row = Row::new();
                (top_row, bot_row)
            }
            5..=10 => {
                let top_row = self.dota_match.players.clone()[0..5]
                    .iter()
                    .fold(Row::new(), |row, player| row.push(player.view())).spacing(row_spacing);
                let bot_row = self.dota_match.players.clone()[5..]
                    .iter()
                    .fold(Row::new(), |row, player| row.push(player.view())).spacing(row_spacing);
                (top_row, bot_row)
            }

            _ => (Row::new(), Row::new()),
        };
        let column: Column<'static, Message> = Column::new().push(top_row).push(bot_row).spacing(col_spacing);

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::Container)
            .into()
    }
}

fn watch() -> impl Future<Output = Vec<i32>> {
    async {
        // let mut log_watcher = LogWatcher::register("log.txt").unwrap();
        // let player_ids = log_watcher.watch().await;
        let player_ids = vec![
            416098293, 926498844, 193296043, 207041414, 218061707, 46333111, 83615933, 346964866,
            244676219, 395739513,
        ];
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
        Self { players: vec![] }
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
        async move {
            info!("In async");
            let mut player = DotaPlayer::new();
            // if it errors return a empty profile to stop wasting api calls
            let response =
                match players_account_id_get(&configuration::Configuration::default(), id).await {
                    Ok(x) => x,
                    // If Player Response is empty or error'd return a default player
                    Err(_) => return player,
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

    // Change to default
    pub fn new() -> Self {
        Self {
            name: String::from("Private Player"),
            image: image::Handle::from_path("resources/default.png"), // replace with empty image?
            image_url: String::from("None"),
            wins: String::from("0"),
            losses: String::from("0"),
        }
    }

    pub fn view(&self) -> Element<'static, Message> {
        let col: Column<'static, Message> = Column::new()
            .push(Text::new(self.name.clone()))
            .push(
                Image::new(self.image.clone())
                    .width(Length::Units(100))
                    .height(Length::Units(100)),
            )
            .push(Text::new(format!("Wins {}", self.wins)))
            .push(Text::new(format!("Loses {}", self.losses)))
            .into();
        Container::new(col).style(style::player_card).into()
    }
}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
}
