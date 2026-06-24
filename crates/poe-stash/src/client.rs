use poe_core::types::{StashItem, StashTab, StashTabColor};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::Instant;

const STASH_API_URL: &str = "https://www.pathofexile.com/character-window/get-stash-items";
const CHARACTER_API_URL: &str = "https://www.pathofexile.com/character-window/get-items";
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
    /// Header-derived delay to apply before the next request (proactive throttle).
    next_delay: Duration,
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
            next_delay: Duration::ZERO,
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
        const MAX_RETRIES: u32 = 3;

        let sessid = self
            .poesessid
            .as_ref()
            .ok_or_else(|| "No POESESSID set".to_string())?
            .clone();

        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("POESESSID={}", sessid))
                .map_err(|e| format!("Invalid POESESSID: {}", e))?,
        );
        headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));

        let mut attempt = 0u32;
        loop {
            // Pace requests: respect the fixed floor and any delay the API's
            // rate-limit headers told us to wait after the previous response.
            if let Some(last) = self.last_request {
                let gap = MIN_REQUEST_INTERVAL.max(self.next_delay);
                let elapsed = last.elapsed();
                if elapsed < gap {
                    tokio::time::sleep(gap - elapsed).await;
                }
            }

            tracing::debug!("Stash API request: {}", url);
            let resp = self
                .http
                .get(url)
                .headers(headers.clone())
                .send()
                .await
                .map_err(|e| format!("Stash API request failed: {}", e))?;

            self.last_request = Some(Instant::now());
            self.next_delay = next_delay_from_headers(resp.headers());
            let status = resp.status();

            // 429: honor Retry-After / rate-limit state and retry instead of failing.
            if status.as_u16() == 429 && attempt < MAX_RETRIES {
                let wait = parse_retry_after(resp.headers())
                    .filter(|d| !d.is_zero())
                    .or_else(|| Some(self.next_delay).filter(|d| !d.is_zero()))
                    .unwrap_or_else(|| Duration::from_secs(2u64.pow(attempt + 1)))
                    .min(Duration::from_secs(60));
                attempt += 1;
                tracing::warn!(
                    "Stash API 429 — backing off {:?} (retry {}/{})",
                    wait,
                    attempt,
                    MAX_RETRIES
                );
                tokio::time::sleep(wait).await;
                continue;
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                tracing::error!("Stash API HTTP {}: {}", status, body);
                return match status.as_u16() {
                    429 => Err("Rate limited — try again in a minute".to_string()),
                    401 | 403 => Err("Authentication failed — check your POESESSID".to_string()),
                    _ => Err(format!("Stash API returned HTTP {}: {}", status, body)),
                };
            }

            return Ok(resp);
        }
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

    /// Fetch a character's items (inventory + equipment) for per-map loot diffing.
    /// Different endpoint than stash, so it has a separate rate budget.
    /// PoE account + character names are alphanumeric/underscore (no spaces), so
    /// no URL-encoding is needed.
    pub async fn fetch_character_inventory(
        &mut self,
        character: &str,
    ) -> Result<Vec<crate::inventory::InventoryItem>, String> {
        let account = self
            .account_name
            .as_ref()
            .ok_or_else(|| "No account name set".to_string())?;
        let url = format!(
            "{}?accountName={}&character={}",
            CHARACTER_API_URL, account, character
        );
        let resp = self.rate_limited_get(&url).await?;

        #[derive(Deserialize)]
        struct Resp {
            #[serde(default)]
            items: Vec<crate::inventory::InventoryItem>,
        }
        let raw: Resp = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse character items: {}", e))?;
        Ok(raw.items)
    }
}

// --- Rate-limit header parsing (GGG returns these on every response) ---

/// Parse a rate-limit header value into `(max_or_current, period_secs, restrict_secs)`
/// triplets. Values look like `"45:60:60"` or `"8:10:10,15:60:60"` (comma-joined rules).
fn parse_triplets(s: &str) -> Vec<(u64, u64, u64)> {
    s.split(',')
        .filter_map(|part| {
            let mut it = part.trim().split(':');
            let a = it.next()?.trim().parse().ok()?;
            let b = it.next()?.trim().parse().ok()?;
            let c = it.next()?.trim().parse().ok()?;
            Some((a, b, c))
        })
        .collect()
}

/// How long to wait before the next request to stay under one rule, given its
/// policy (`max:period:restrict`) and state (`current:period:active_restrict`)
/// header values (each may hold several comma-joined rules; the strictest wins).
fn rule_delay(policy: &str, state: &str) -> Duration {
    let pol = parse_triplets(policy);
    let st = parse_triplets(state);
    let mut wait = Duration::ZERO;
    for ((max, period, _), (cur, _, active)) in pol.iter().zip(st.iter()) {
        let d = if *active > 0 {
            Duration::from_secs(*active) // a restriction is currently active
        } else if *max > 0 && *cur >= *max {
            Duration::from_secs(*period) // window full → wait it out
        } else if *max > 0 && *cur + 1 >= *max {
            Duration::from_secs(*period) / (*max as u32) // pace the final slot
        } else {
            Duration::ZERO
        };
        if d > wait {
            wait = d;
        }
    }
    wait
}

/// Largest required delay across the account/ip/client rate-limit rules.
fn next_delay_from_headers(headers: &HeaderMap) -> Duration {
    let mut wait = Duration::ZERO;
    for rule in ["account", "ip", "client"] {
        let policy = headers.get(format!("x-rate-limit-{rule}").as_str());
        let state = headers.get(format!("x-rate-limit-{rule}-state").as_str());
        if let (Some(p), Some(s)) = (policy, state) {
            if let (Ok(p), Ok(s)) = (p.to_str(), s.to_str()) {
                let d = rule_delay(p, s);
                if d > wait {
                    wait = d;
                }
            }
        }
    }
    wait
}

/// Parse `Retry-After` (delta-seconds form) into a Duration.
fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    let v = headers.get(reqwest::header::RETRY_AFTER)?.to_str().ok()?;
    v.trim().parse::<u64>().ok().map(Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_delay_headroom_is_zero() {
        // 2/45 hits used against a 45-per-60s rule → plenty of room.
        assert_eq!(rule_delay("45:60:60", "2:60:0"), Duration::ZERO);
    }

    #[test]
    fn rule_delay_active_restriction_waits() {
        // active restriction of 12s in the state's third field.
        assert_eq!(rule_delay("45:60:60", "46:60:12"), Duration::from_secs(12));
    }

    #[test]
    fn rule_delay_full_window_waits_period() {
        // at the cap, no active restriction → wait the period out.
        assert_eq!(rule_delay("5:10:30", "5:10:0"), Duration::from_secs(10));
    }

    #[test]
    fn rule_delay_strictest_rule_wins() {
        // two rules; the second is at its cap → its period dominates.
        assert_eq!(
            rule_delay("8:10:10,15:60:60", "1:10:0,15:60:0"),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn retry_after_parsed() {
        let mut h = HeaderMap::new();
        h.insert(reqwest::header::RETRY_AFTER, HeaderValue::from_static("17"));
        assert_eq!(parse_retry_after(&h), Some(Duration::from_secs(17)));
        assert_eq!(parse_retry_after(&HeaderMap::new()), None);
    }

    #[test]
    fn next_delay_reads_account_rule() {
        let mut h = HeaderMap::new();
        h.insert("x-rate-limit-account", HeaderValue::from_static("5:10:30"));
        h.insert("x-rate-limit-account-state", HeaderValue::from_static("5:10:0"));
        assert_eq!(next_delay_from_headers(&h), Duration::from_secs(10));
    }
}
