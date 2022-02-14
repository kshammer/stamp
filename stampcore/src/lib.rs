use url::Url;
use tracing::{event, Level, span, info};
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

#[derive(Debug)]
pub struct OpenDotaAPI{
    endpoint: Endpoint
}


impl OpenDotaAPI{
    pub fn new() -> OpenDotaAPI{
        OpenDotaAPI{
            endpoint: Endpoint::Players
        }
    }

    pub fn endpoint(&mut self, endpoint:Endpoint) -> &mut OpenDotaAPI{
        self.endpoint = endpoint;
        self
    }

    fn prepare_url(&self) -> Result<String, OpenDotaAPIError> {
        let mut url = Url::parse(BASE_URL)?;
        url.path_segments_mut().unwrap().push(&self.endpoint.to_string()).push("83615933");
        Ok(url.to_string())
    }

    pub fn fetch(&self) -> Result<String, OpenDotaAPIError>{
        let span = span!(Level::TRACE, "api_resquest");
        let _enter = span.enter();
        info!("Hi");
        let url = self.prepare_url()?;
        info!("URL {:?}", url);
        let req = ureq::get(&url);
        let response: String = req.call()?.into_string()?;
        info!("Repsonse {:?}", response);
        return Ok(response)
    }
    
}