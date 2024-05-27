use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::log::debug;

use crate::{
    obligato_client::Client,
    types::{ObligatoObligation, ObligatoSetOff},
};

pub struct HttpClient {
    client: reqwest::Client,
    url: Url,
    key: String,
}

impl HttpClient {
    pub fn new(url: Url, key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            key,
        }
    }

    fn url_with_path(&self, path: &str) -> Url {
        let mut url = self.url.clone();
        url.set_path(path);
        url
    }
}

#[async_trait]
impl Client for HttpClient {
    type Error = reqwest::Error;

    async fn get_obligations(&self) -> Result<Vec<ObligatoObligation>, Self::Error> {
        let response = self
            .client
            .post(self.url_with_path("api/sync/obligations2contract"))
            .json(&json!({"denom_id": "1", "key": self.key }))
            .send()
            .await?
            .json::<GetObligationsResponse>()
            .await?;

        Ok(response.all_obligations.obligations)
    }

    async fn set_setoffs(&self, setoffs: Vec<ObligatoSetOff>) -> Result<(), Self::Error> {
        let response = self
            .client
            .post(self.url_with_path("api/set-offs"))
            .json(&setoffs)
            .send()
            .await?;
        debug!("{}", response.text().await.unwrap());

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GetObligationsInnerResponse {
    obligations: Vec<ObligatoObligation>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GetObligationsResponse {
    #[serde(rename = "allObligations")]
    all_obligations: GetObligationsInnerResponse,
}
