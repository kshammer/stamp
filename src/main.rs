#![feature(windows_by_handle)]
use iced::{
    image, Application, Clipboard, Column, Command, Container, Element, Image, Length, Row,
    Settings, Text,
};
use opendota_client::apis::{
    configuration, players_api::players_account_id_get, players_api::players_account_id_wl_get,
};
use opendota_client::models::PlayerResponseProfile;
use opendota_client::models::{player_response::PlayerResponse, PlayerWinLossResponse};
use reqwest;
use std::future::Future;


mod log_watch;
pub fn main() -> iced::Result {
    Stamp::run(Settings::default())
}

#[derive(Default)]
struct Stamp {
    dota_match: DotaMatch
}

#[derive(Debug, Clone)]
enum Message {
    LookingForMatch,
    PlayerFound(i32),
}

impl Application for Stamp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self{
                ..Self::default()
            },
            Command::none()
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
            Message::LookingForMatch => {
                Command::none()
            }
            Message::PlayerFound(player_id) => {

                return Command::perform(DotaPlayer::fetch_player_info(player_id), move |_message| {
                    Message::LookingForMatch
                })
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = Column::new().push(
            self.dota_match.view().map(move |_message| Message::LookingForMatch)
        );
        Container::new(content).width(Length::Fill).height(Length::Fill).into()
    }
}

async fn watch() -> Result<Vec<i32>, Error> {
    // let mut log_watcher = LogWatcher::register("log.txt").unwrap();
    // let player_ids = log_watcher.watch().await;
    let player_ids = vec![83615933, 244676219];
    Ok(player_ids)
}

#[derive(Debug, Clone)]
struct DotaMatch {
    player_ids: Vec<DotaPlayer>,
}

impl Default for DotaMatch{
    fn default() -> Self {
        Self::new()
    }
}

impl DotaMatch {
    pub fn new() -> Self {
        Self { player_ids: vec![] }
    }

    // async fn create_players(ids: Vec<i32>) -> DotaMatch {
    //     let mut dota_match = DotaMatch::new();
    //     for id in ids.iter() {
    //         dota_match
    //             .player_ids
    //             .push(DotaPlayer::fetch_player_info(*id).await)
    //     }
    //     dota_match
    // }

    // fn view(&mut self) -> Element<Message> {
    //     let mut elements = Vec::<Element<Message>>::new();
    //     for player in self.players.clone() {
    //         elements.push(DotaPlayer::view(player));
    //     }
    //     Row::with_children(elements).into()
    // }

    fn view(&mut self) -> Element<Message> {
        let mut elements = Vec::<Element<Message>>::new();
            for player in self.player_ids.clone() {
                elements.push(Text::new(player.name.to_string()).into());
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

    pub fn fetch_player_info(id: i32) -> impl Future<Output = DotaPlayer> {
        async move {
            let mut player = DotaPlayer::new();
            let response =
                match players_account_id_get(&configuration::Configuration::default(), id).await {
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
    
            player
        }
        
    }

    async fn fetch_player_image(url: String) -> Result<image::Handle, reqwest::Error> {
        let bytes = reqwest::get(&url).await?.bytes().await?;
        Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
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
            .push(Text::new(format!("Loses {}", &player.losses)))
            .into()
    }

}

#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
}
