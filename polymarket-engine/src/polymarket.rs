use reqwest::Client;
use serde_json::Value;
use tracing::{info, warn};

use crate::config::Config;
use crate::error::{EngineError, EngineResult};
use crate::types::*;

/// Polymarket CLOB + Gamma API client
pub struct PolymarketClient {
    http: Client,
    clob_url: String,
    gamma_url: String,
    _chain_id: u64,
    timeout: std::time::Duration,
    max_retries: u32,
}

impl PolymarketClient {
    pub fn new(config: &Config) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .connect_timeout(std::time::Duration::from_secs(10))
                .pool_max_idle_per_host(5)
                .build()
                .unwrap_or_else(|_| Client::new()),
            clob_url: config.clob_url.clone(),
            gamma_url: config.gamma_url.clone(),
            _chain_id: config.chain_id,
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
        }
    }

    /// Execute HTTP GET with retries and error classification
    async fn get_with_retry(&self, url: &str, query: &[(&str, String)]) -> EngineResult<Value> {
        let mut last_err = None;

        for attempt in 0..self.max_retries {
            if attempt > 0 {
                let delay = 1000 * 2u64.pow(attempt);
                tracing::debug!("Retry {}/{} for {} (delay {}ms)", attempt + 1, self.max_retries, url, delay);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }

            match self.http.get(url).query(query).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();

                    if status == 429 {
                        let retry_after = resp
                            .headers()
                            .get("retry-after")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(5)
                            * 1000;
                        last_err = Some(EngineError::PolymarketRateLimit { retry_after_ms: retry_after });
                        tokio::time::sleep(std::time::Duration::from_millis(retry_after)).await;
                        continue;
                    }

                    if !resp.status().is_success() {
                        let body = resp.text().await.unwrap_or_default();
                        last_err = Some(EngineError::PolymarketApi {
                            status,
                            message: body.chars().take(200).collect(),
                        });
                        if status >= 500 { continue; } // Retry 5xx
                        return Err(last_err.unwrap());
                    }

                    return resp.json().await.map_err(|e| {
                        EngineError::PolymarketParse(format!("JSON decode: {}", e))
                    });
                }
                Err(e) => {
                    if e.is_timeout() {
                        last_err = Some(EngineError::PolymarketTimeout(self.timeout.as_secs()));
                    } else if e.is_connect() {
                        last_err = Some(EngineError::ConnectionFailed {
                            url: url.to_string(),
                            reason: e.to_string(),
                        });
                    } else {
                        last_err = Some(EngineError::Http(e));
                    }
                    continue;
                }
            }
        }

        Err(last_err.unwrap_or_else(|| EngineError::Internal("Unknown retry exhaustion".into())))
    }

    // ═══════════════════════════════════════
    // GAMMA MARKETS API
    // ═══════════════════════════════════════

    /// Fetch active markets with filters
    pub async fn get_markets(
        &self,
        limit: u32,
        offset: u32,
        active: bool,
    ) -> EngineResult<Vec<Market>> {
        let url = format!("{}/markets", self.gamma_url);

        let data = self.get_with_retry(&url, &[
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
            ("active", active.to_string()),
            ("closed", "false".to_string()),
            ("order", "volume24hr".to_string()),
            ("ascending", "false".to_string()),
        ]).await?;

        let markets_raw = data.as_array().ok_or_else(|| {
            EngineError::PolymarketParse("Expected array from /markets".into())
        })?;

        let mut result = Vec::new();
        let mut parse_errors = 0;

        for m in markets_raw {
            match parse_market(m) {
                Ok(market) => result.push(market),
                Err(_) => { parse_errors += 1; }
            }
        }

        if parse_errors > 0 {
            warn!("Skipped {} unparseable markets out of {}", parse_errors, markets_raw.len());
        }

        info!("Fetched {} markets from Gamma API", result.len());
        Ok(result)
    }

    /// Get a single market by slug
    pub async fn get_market_by_slug(&self, slug: &str) -> EngineResult<Option<Market>> {
        let url = format!("{}/markets", self.gamma_url);

        let data = self.get_with_retry(&url, &[
            ("slug", slug.to_string()),
        ]).await?;

        let markets = data.as_array().ok_or_else(|| {
            EngineError::PolymarketParse("Expected array from /markets?slug=".into())
        })?;

        if let Some(m) = markets.first() {
            Ok(Some(parse_market(m)?))
        } else {
            Ok(None)
        }
    }

    /// Get a single market by market ID
    pub async fn get_market_by_id(&self, market_id: &str) -> EngineResult<Option<Market>> {
        let url = format!("{}/markets", self.gamma_url);

        let data = self.get_with_retry(&url, &[
            ("id", market_id.to_string()),
        ]).await?;

        let markets = data.as_array().ok_or_else(|| {
            EngineError::PolymarketParse("Expected array from /markets?id=".into())
        })?;

        if let Some(m) = markets.first() {
            Ok(Some(parse_market(m)?))
        } else {
            Ok(None)
        }
    }

    /// Get high-volume tradeable markets
    pub async fn scan_opportunities(
        &self,
        min_volume_24h: f64,
        min_liquidity: f64,
    ) -> EngineResult<Vec<Market>> {
        let markets = self.get_markets(100, 0, true).await?;

        let filtered: Vec<Market> = markets
            .into_iter()
            .filter(|m| {
                let vol_ok = m.volume_24hr.unwrap_or(0.0) >= min_volume_24h;
                let liq_ok = m.liquidity.unwrap_or(0.0) >= min_liquidity;
                let orderbook = m.enable_order_book.unwrap_or(false);
                let has_tokens = m.clob_token_ids.is_some();
                vol_ok && liq_ok && orderbook && has_tokens
            })
            .collect();

        info!("Found {} tradeable opportunities (vol>={}, liq>={})",
            filtered.len(), min_volume_24h, min_liquidity);
        Ok(filtered)
    }

    // ═══════════════════════════════════════
    // CLOB ORDER API
    // ═══════════════════════════════════════

    /// Get CLOB API server status
    pub async fn get_server_status(&self) -> EngineResult<Value> {
        let url = format!("{}/", self.clob_url);
        self.get_with_retry(&url, &[]).await
    }

    /// Get order book for a token
    pub async fn get_order_book(&self, token_id: &str) -> EngineResult<Value> {
        let url = format!("{}/book", self.clob_url);
        self.get_with_retry(&url, &[("token_id", token_id.to_string())]).await
    }

    /// Get midpoint price for a token
    pub async fn get_midpoint(&self, token_id: &str) -> EngineResult<f64> {
        let url = format!("{}/midpoint", self.clob_url);
        let data = self.get_with_retry(&url, &[("token_id", token_id.to_string())]).await?;

        data["mid"]
            .as_str()
            .ok_or_else(|| EngineError::PolymarketParse("Missing 'mid' in midpoint response".into()))?
            .parse::<f64>()
            .map_err(|e| EngineError::PolymarketParse(format!("Invalid midpoint: {}", e)))
    }

    /// Get spread for a token
    pub async fn get_spread(&self, token_id: &str) -> EngineResult<(f64, f64)> {
        let url = format!("{}/spread", self.clob_url);
        let data = self.get_with_retry(&url, &[("token_id", token_id.to_string())]).await?;

        let bid = data["bid"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| EngineError::PolymarketParse("Invalid bid in spread".into()))?;
        let ask = data["ask"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| EngineError::PolymarketParse("Invalid ask in spread".into()))?;
        Ok((bid, ask))
    }

    /// Get price history for a market
    pub async fn get_price_history(
        &self,
        token_id: &str,
        interval: &str,
        fidelity: u32,
    ) -> EngineResult<Vec<Value>> {
        let url = format!("{}/prices-history", self.clob_url);
        let data = self.get_with_retry(&url, &[
            ("market", token_id.to_string()),
            ("interval", interval.to_string()),
            ("fidelity", fidelity.to_string()),
        ]).await?;

        Ok(data["history"]
            .as_array()
            .cloned()
            .unwrap_or_default())
    }

    /// Get recent trades for market
    pub async fn get_trades(&self, condition_id: &str) -> EngineResult<Vec<Value>> {
        let url = format!("{}/trades", self.clob_url);
        let data = self.get_with_retry(&url, &[("condition_id", condition_id.to_string())]).await?;
        Ok(data.as_array().cloned().unwrap_or_default())
    }

    // ═══════════════════════════════════════
    // ORDER PLACEMENT (requires auth)
    // ═══════════════════════════════════════

    /// Place an order (simulated - needs full EIP712 signing)
    pub async fn place_order(&self, order: &OrderRequest) -> EngineResult<OrderResponse> {
        // NOTE: Full order placement requires EIP712 signing
        // Real implementation needs:
        // 1. API key derivation via CLOB auth
        // 2. EIP712 order struct signing
        // 3. POST to /order endpoint

        warn!(
            "ORDER INTENT: {} {} @ {} (token: {})",
            match order.side { Side::Buy => "BUY", Side::Sell => "SELL" },
            order.size,
            order.price,
            order.token_id
        );

        // Simulate success for dashboard
        Ok(OrderResponse {
            success: true,
            order_id: Some(uuid::Uuid::new_v4().to_string()),
            error_msg: None,
            transaction_hash: None,
        })
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> EngineResult<bool> {
        warn!("CANCEL ORDER: {}", order_id);
        Ok(true)
    }

    // ═══════════════════════════════════════
    // ARBITRAGE SCANNING
    // ═══════════════════════════════════════

    /// Get best ask prices for both YES and NO tokens of a binary market.
    /// Returns (yes_ask, no_ask) from the actual CLOB orderbook.
    pub async fn get_binary_prices(&self, yes_token_id: &str, no_token_id: &str) -> EngineResult<(f64, f64)> {
        // Fetch both orderbooks
        let yes_book = self.get_order_book(yes_token_id).await?;
        let no_book = self.get_order_book(no_token_id).await?;

        let yes_ask = Self::extract_best_ask(&yes_book)
            .ok_or_else(|| EngineError::PolymarketParse("No YES asks in orderbook".into()))?;
        let no_ask = Self::extract_best_ask(&no_book)
            .ok_or_else(|| EngineError::PolymarketParse("No NO asks in orderbook".into()))?;

        Ok((yes_ask, no_ask))
    }

    /// Get best bid prices for both YES and NO tokens.
    /// Returns (yes_bid, no_bid).
    pub async fn get_binary_bids(&self, yes_token_id: &str, no_token_id: &str) -> EngineResult<(f64, f64)> {
        let yes_book = self.get_order_book(yes_token_id).await?;
        let no_book = self.get_order_book(no_token_id).await?;

        let yes_bid = Self::extract_best_bid(&yes_book)
            .ok_or_else(|| EngineError::PolymarketParse("No YES bids in orderbook".into()))?;
        let no_bid = Self::extract_best_bid(&no_book)
            .ok_or_else(|| EngineError::PolymarketParse("No NO bids in orderbook".into()))?;

        Ok((yes_bid, no_bid))
    }

    /// Fetch all markets for a given event (for multi-outcome arb scanning).
    /// Uses Gamma API events endpoint to get related markets.
    pub async fn get_event_markets(&self, event_slug: &str) -> EngineResult<Vec<Market>> {
        let url = format!("{}/events", self.gamma_url);
        let data = self.get_with_retry(&url, &[("slug", event_slug.to_string())]).await?;

        // Events API returns array or single object
        let event = if let Some(arr) = data.as_array() {
            arr.first().cloned()
        } else {
            Some(data)
        };

        let event = event.ok_or_else(|| {
            EngineError::PolymarketParse(format!("Event not found: {}", event_slug))
        })?;

        let markets_raw = event["markets"].as_array().ok_or_else(|| {
            EngineError::PolymarketParse("Event has no 'markets' array".into())
        })?;

        let mut result = Vec::new();
        for m in markets_raw {
            if let Ok(market) = parse_market(m) {
                if market.enable_order_book.unwrap_or(false) {
                    result.push(market);
                }
            }
        }

        info!("Event '{}': found {} tradeable markets", event_slug, result.len());
        Ok(result)
    }

    /// Scan for neg-risk multi-outcome events among cached markets.
    /// Returns event slugs that have neg_risk=true and multiple markets.
    pub async fn get_neg_risk_events(&self, limit: u32) -> EngineResult<Vec<Value>> {
        let url = format!("{}/events", self.gamma_url);
        let data = self.get_with_retry(&url, &[
            ("limit", limit.to_string()),
            ("active", "true".to_string()),
            ("closed", "false".to_string()),
        ]).await?;

        let events = data.as_array().ok_or_else(|| {
            EngineError::PolymarketParse("Expected array from /events".into())
        })?;

        let neg_risk_events: Vec<Value> = events
            .iter()
            .filter(|e| {
                let is_neg = e["negRisk"].as_bool().unwrap_or(false);
                let has_markets = e["markets"].as_array().map(|m| m.len() >= 2).unwrap_or(false);
                is_neg && has_markets
            })
            .cloned()
            .collect();

        info!("Found {} neg-risk multi-outcome events", neg_risk_events.len());
        Ok(neg_risk_events)
    }

    /// Get available liquidity (depth) at best ask for a token.
    /// Returns (best_ask_price, size_available_at_best_ask).
    pub async fn get_ask_depth(&self, token_id: &str) -> EngineResult<(f64, f64)> {
        let book = self.get_order_book(token_id).await?;
        let asks = book["asks"].as_array().ok_or_else(|| {
            EngineError::PolymarketParse("No asks array in orderbook".into())
        })?;

        if asks.is_empty() {
            return Err(EngineError::PolymarketParse("Asks array is empty".into()));
        }

        // Asks are sorted lowest first
        let best = &asks[0];
        let price = Self::parse_book_price(best)?;
        let size = Self::parse_book_size(best)?;

        Ok((price, size))
    }

    /// Get available liquidity (depth) at best bid for a token.
    /// Returns (best_bid_price, size_available_at_best_bid).
    pub async fn get_bid_depth(&self, token_id: &str) -> EngineResult<(f64, f64)> {
        let book = self.get_order_book(token_id).await?;
        let bids = book["bids"].as_array().ok_or_else(|| {
            EngineError::PolymarketParse("No bids array in orderbook".into())
        })?;

        if bids.is_empty() {
            return Err(EngineError::PolymarketParse("Bids array is empty".into()));
        }

        // Bids are sorted highest first
        let best = &bids[0];
        let price = Self::parse_book_price(best)?;
        let size = Self::parse_book_size(best)?;

        Ok((price, size))
    }

    // ═══════════════════════════════════════
    // ORDERBOOK HELPERS
    // ═══════════════════════════════════════

    fn extract_best_ask(book: &Value) -> Option<f64> {
        book["asks"]
            .as_array()?
            .first()
            .and_then(|a| Self::parse_book_price(a).ok())
    }

    fn extract_best_bid(book: &Value) -> Option<f64> {
        book["bids"]
            .as_array()?
            .first()
            .and_then(|b| Self::parse_book_price(b).ok())
    }

    fn parse_book_price(entry: &Value) -> EngineResult<f64> {
        entry["price"]
            .as_str()
            .or_else(|| entry["p"].as_str())
            .ok_or_else(|| EngineError::PolymarketParse("No price in book entry".into()))?
            .parse::<f64>()
            .map_err(|e| EngineError::PolymarketParse(format!("Invalid book price: {}", e)))
    }

    fn parse_book_size(entry: &Value) -> EngineResult<f64> {
        entry["size"]
            .as_str()
            .or_else(|| entry["s"].as_str())
            .ok_or_else(|| EngineError::PolymarketParse("No size in book entry".into()))?
            .parse::<f64>()
            .map_err(|e| EngineError::PolymarketParse(format!("Invalid book size: {}", e)))
    }
}

/// Parse Gamma API JSON into Market struct
pub fn parse_market(v: &Value) -> EngineResult<Market> {
    let id = v["id"].as_str().unwrap_or("").to_string();
    if id.is_empty() {
        return Err(EngineError::PolymarketParse("Market missing 'id' field".into()));
    }

    Ok(Market {
        id,
        question: v["question"].as_str().map(String::from),
        condition_id: v["conditionId"].as_str().map(String::from),
        slug: v["slug"].as_str().map(String::from),
        end_date: v["endDate"].as_str().map(String::from),
        category: v["category"].as_str().map(String::from),
        volume: v["volumeNum"].as_f64().or_else(|| {
            v["volume"].as_str().and_then(|s| s.parse().ok())
        }),
        liquidity: v["liquidityNum"].as_f64().or_else(|| {
            v["liquidity"].as_str().and_then(|s| s.parse().ok())
        }),
        outcome_prices: v["outcomePrices"].as_str().map(String::from),
        outcomes: v["outcomes"].as_str().map(String::from),
        active: v["active"].as_bool(),
        clob_token_ids: v["clobTokenIds"].as_str().map(String::from),
        enable_order_book: v["enableOrderBook"].as_bool(),
        best_bid: v["bestBid"].as_f64(),
        best_ask: v["bestAsk"].as_f64(),
        last_trade_price: v["lastTradePrice"].as_f64(),
        volume_24hr: v["volume24hr"].as_f64(),
        neg_risk: v["negRisk"].as_bool().or_else(|| {
            v["events"]
                .as_array()
                .and_then(|e| e.first())
                .and_then(|e| e["negRisk"].as_bool())
        }),
        order_price_min_tick_size: v["orderPriceMinTickSize"].as_f64(),
    })
}
