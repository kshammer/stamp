use iced::{
    image, Application, Clipboard, Column, Command, Container, Element, Image, Length, Settings,
    Text
};
use opendota_client::apis::{configuration, players_api::players_account_id_get};
use opendota_client::models::player_response::PlayerResponse;
use reqwest;

pub fn main() -> iced::Result {
    Stamp::run(Settings::default())
}

#[derive(Debug)]
enum Stamp {
    Loading,
    Loaded { player: DotaPlayer },
}

#[derive(Debug, Clone)]
enum Message {
    DotaPlayerFound(Result<DotaPlayer, Error>),
}

impl Application for Stamp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Stamp, Command<Self::Message>) {
        (
            Stamp::Loading,
            Command::perform(DotaPlayer::search(), Message::DotaPlayerFound),
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
            Message::DotaPlayerFound(Ok(dotaplayer)) => {
                *self = Stamp::Loaded { player: dotaplayer };
                Command::none()
            }
            Message::DotaPlayerFound(Err(_error)) => Command::none(),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = match self {
            Stamp::Loading => Column::new().push(Text::new("Searching for Dota Players")),
            Stamp::Loaded { player } => Column::new().push(player.view()),
        };
        Container::new(content).into()
    }

}

#[derive(Debug, Clone)]
struct DotaPlayer {
    name: String,
    image: image::Handle,
    image_url: String, 
}

impl DotaPlayer {
    fn view(&mut self) -> Column<Message> {
        Column::new().push(Text::new(self.name.clone())).push(
            Image::new(self.image.clone())
                .width(Length::Units(100))
                .height(Length::Units(100)),
        )
    }

    async fn search() -> Result<DotaPlayer, Error> {
        let response = match players_account_id_get(
            &configuration::Configuration::default(),
            83615933,
        )
        .await
        {
            Ok(x) => x,
            Err(_) => {
                PlayerResponse {
                    tracked_until: None,
                    solo_competitive_rank: None,
                    competitive_rank: None,
                    rank_tier: None,
                    leaderboard_rank: None,
                    mmr_estimate: None,
                    profile: None,
                }
            }
        };
        let profile = response.profile.unwrap();
        let profile_name = *profile;
        let mut player = DotaPlayer {
            name: match profile_name.personaname {
                Some(x) => x,
                None => "No name".to_string(),
            },
            image: image::Handle::from_path("resources/default.png"),

            image_url: match profile_name.avatarfull {
                Some(x) => x, 
                None => "No url".to_string()
            }
            
        };
        player.image = match Self::fetch_player_image(player.image_url.clone()).await{
            Ok(x) => x,
            Err(_) => image::Handle::from_path("resources/default.png")
        };
        Ok(player)
    }

    async fn fetch_player_image(url: String) -> Result<image::Handle, reqwest::Error>{
        let bytes = reqwest::get(&url).await?.bytes().await?;
        Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))

    }

}


#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
}
