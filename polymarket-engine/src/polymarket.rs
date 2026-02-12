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
}

/// Parse Gamma API JSON into Market struct
fn parse_market(v: &Value) -> EngineResult<Market> {
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
