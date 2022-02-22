use url::Url;
use tracing::{Level, span, info};
use serde::Deserialize;
use serde_json::{Value};

const BASE_URL: &str = "https://api.opendota.com/api";

#[derive(thiserror::Error, Debug)]
pub enum OpenDotaAPIError {
    #[error("Url parsing failed")]
    UrlParsing(#[from] url::ParseError),
    #[error("Failed fetching articles")]
    RequestFailed(#[from] ureq::Error),
    #[error("Failed converting response to string")]
    FailedResponseToString(#[from] std::io::Error),
}
#[derive(Debug)]
pub enum Endpoint{
    Players
}
impl ToString for Endpoint{
    fn to_string(&self) -> String {
        match self {
            Self::Players => "players".to_string()
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PlayersAPIResponse{
    pub account_id: String, 
    pub username: String, 
    pub profile_picture: String
}

impl PlayersAPIResponse{
    pub fn account_id(&self) -> &str {
        &self.account_id
    }
    pub fn username(&self) -> &str{
        &self.username
    }
    pub fn profile_picture(&self) -> &str{
        &self.profile_picture
    }
}

#[derive(Debug)]
pub struct OpenDotaAPI{}


impl OpenDotaAPI{

    fn prepare_player_url(endpoint: &str, account_id: &str) -> Result<String, OpenDotaAPIError> {
        let mut url = Url::parse(BASE_URL)?;
        url.path_segments_mut().unwrap().push(endpoint).push(account_id);
        Ok(url.to_string())
    }

    pub fn fetch_player(user_id: &str) -> Result<PlayersAPIResponse, OpenDotaAPIError>{
        let span = span!(Level::TRACE, "player_request");
        let _enter = span.enter();
        let url = Self::prepare_player_url("players", user_id)?;
        info!("URL {:?}", url);
        let req = ureq::get(&url);
        let response: String = req.call()?.into_string()?;
        info!("Repsonse {:?}", response);
        return Ok(Self::parse_player_object(&response));
    }

    // Does this method need &self ??/ 
    fn parse_player_object(response: &str) -> PlayersAPIResponse{
        let v: Value = serde_json::from_str(response).unwrap(); // weird champ
        PlayersAPIResponse{
            account_id: v["profile"]["account_id"].to_string(),
            username:  v["profile"]["personaname"].to_string(),
            profile_picture:  v["avatarmedium"].to_string()
        }

    }
    
}