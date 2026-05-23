use poe_core::types::{StashItem, StashTab, StashTabColor};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::Instant;

const STASH_API_URL: &str = "https://www.pathofexile.com/character-window/get-stash-items";
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(1100);
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36";

#[derive(Deserialize)]
struct RawStashResponse {
    #[serde(default)]
    tabs: Vec<RawTab>,
    #[serde(default)]
    items: Vec<RawItem>,
}

#[derive(Deserialize)]
struct RawTab {
    n: String,
    i: u32,
    #[serde(rename = "type")]
    tab_type: String,
    colour: Option<RawColour>,
}

#[derive(Deserialize)]
struct RawColour {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Deserialize)]
struct RawItem {
    #[serde(default)]
    name: String,
    #[serde(rename = "typeLine")]
    type_line: String,
    #[serde(rename = "baseType", default)]
    base_type: Option<String>,
    #[serde(rename = "stackSize", default)]
    stack_size: Option<u32>,
    #[serde(rename = "maxStackSize", default)]
    max_stack_size: Option<u32>,
    icon: String,
    #[serde(default)]
    ilvl: Option<u32>,
    #[serde(default)]
    identified: Option<bool>,
    #[serde(rename = "frameType", default)]
    frame_type: Option<u32>,
}

pub struct StashClient {
    http: reqwest::Client,
    poesessid: Option<String>,
    account_name: Option<String>,
    last_request: Option<Instant>,
}

impl StashClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .user_agent(BROWSER_UA)
                .build()
                .expect("failed to build HTTP client"),
            poesessid: None,
            account_name: None,
            last_request: None,
        }
    }

    pub fn set_credentials(&mut self, poesessid: String, account_name: String) {
        self.poesessid = Some(poesessid);
        self.account_name = Some(account_name);
    }

    pub fn clear_credentials(&mut self) {
        self.poesessid = None;
        self.account_name = None;
    }

    pub fn is_configured(&self) -> bool {
        self.poesessid.is_some() && self.account_name.is_some()
    }

    async fn rate_limited_get(&mut self, url: &str) -> Result<reqwest::Response, String> {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < MIN_REQUEST_INTERVAL {
                tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
            }
        }

        let sessid = self
            .poesessid
            .as_ref()
            .ok_or_else(|| "No POESESSID set".to_string())?;

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("POESESSID={}", sessid))
                .map_err(|e| format!("Invalid POESESSID: {}", e))?,
        );
        headers.insert(
            "X-Requested-With",
            HeaderValue::from_static("XMLHttpRequest"),
        );

        tracing::debug!("Stash API request: {}", url);
        let resp = self
            .http
            .get(url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("Stash API request failed: {}", e))?;

        self.last_request = Some(Instant::now());

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("Stash API HTTP {}: {}", status, body);
            return match status.as_u16() {
                429 => Err("Rate limited — try again in a minute".to_string()),
                401 | 403 => Err("Authentication failed — check your POESESSID".to_string()),
                _ => Err(format!("Stash API returned HTTP {}: {}", status, body)),
            };
        }

        Ok(resp)
    }

    pub async fn validate_session(&mut self) -> Result<(), String> {
        let url = format!("{}?league=Standard&tabs=1&tabIndex=0", STASH_API_URL);
        self.rate_limited_get(&url).await?;
        Ok(())
    }

    pub async fn fetch_tabs(&mut self, league: &str) -> Result<Vec<StashTab>, String> {
        let url = format!(
            "{}?league={}&tabs=1&tabIndex=0",
            STASH_API_URL, league
        );

        let resp = self.rate_limited_get(&url).await?;
        let raw: RawStashResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse stash tabs: {}", e))?;

        Ok(raw.tabs.into_iter().map(|t| StashTab {
            id: t.n,
            index: t.i,
            tab_type: t.tab_type,
            color: t.colour.map(|c| StashTabColor {
                r: c.r,
                g: c.g,
                b: c.b,
            }),
        }).collect())
    }

    pub async fn fetch_tab_items(
        &mut self,
        league: &str,
        tab_index: u32,
    ) -> Result<Vec<StashItem>, String> {
        let url = format!(
            "{}?league={}&tabIndex={}",
            STASH_API_URL, league, tab_index
        );

        let resp = self.rate_limited_get(&url).await?;
        let raw: RawStashResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse stash items: {}", e))?;

        Ok(raw.items.into_iter().map(|i| {
            let name = i.name.replace("<<set:MS>><<set:M>><<set:S>>", "").trim().to_string();
            StashItem {
                name,
                type_line: i.type_line,
                base_type: i.base_type,
                stack_size: i.stack_size,
                max_stack_size: i.max_stack_size,
                icon: i.icon,
                ilvl: i.ilvl,
                identified: i.identified,
                frame_type: i.frame_type,
            }
        }).collect())
    }
}
