use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    obligato_client::Client,
    types::{ObligatoObligation, ObligatoSetOff},
};

pub struct HttpClient {
    client: reqwest::Client,
    url: Url,
}

impl HttpClient {
    pub fn new(url: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
        }
    }

    fn url_with_path(&self, path: &str) -> Url {
        let mut url = self.url.clone();
        url.set_path(path);
        url
    }
}

// eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImNhZXNzdGpjdG16bXVqaW55cGJlIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MDMwMTkxNDksImV4cCI6MjAxODU5NTE0OX0.gQsQ26W2AGfwmncTecfKaenRaxhXwZFqLtmcllD_7Sg

#[async_trait]
impl Client for HttpClient {
    type Error = reqwest::Error;

    async fn get_obligations(&self) -> Result<Vec<ObligatoObligation>, Self::Error> {
        let response = self
            .client
            .post(self.url_with_path("api/sync/obligations2contract"))
            .json(&json!({"denom_id": "1", "key": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImNhZXNzdGpjdG16bXVqaW55cGJlIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTcwMzAxOTE0OSwiZXhwIjoyMDE4NTk1MTQ5fQ.EV6v5J3dz8WHAdTK4_IEisKzF-n1Gqyn4wCce_Zrqf4"}))
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
        println!("{}", response.text().await.unwrap());

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
