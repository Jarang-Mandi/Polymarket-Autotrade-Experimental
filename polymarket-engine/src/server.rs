use std::sync::Arc;
use std::net::SocketAddr;
use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::IntoResponse,
    routing::{get, post},
    Router,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use tokio::time::{self, Duration};
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, warn};

use crate::engine::{TradingEngine, EngineState};
use crate::error::EngineError;
use crate::types::*;

type SharedState = Arc<RwLock<EngineState>>;
type SharedEngine = Arc<TradingEngine>;

/// Start the dashboard HTTP + WebSocket server
pub async fn start_dashboard_server(
    host: &str,
    port: u16,
    state: SharedState,
    engine: SharedEngine,
) -> Result<(), EngineError> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Read endpoints (dashboard + OpenClaw)
        .route("/api/state", get(get_state))
        .route("/api/positions", get(get_positions))
        .route("/api/trades", get(get_trades))
        .route("/api/markets", get(get_markets))
        .route("/api/costs", get(get_costs))
        .route("/api/health", get(health))
        // Command endpoints (OpenClaw → Engine)
        .route("/api/trade", post(post_trade))
        .route("/api/close", post(post_close))
        .route("/api/report-cost", post(post_report_cost))
        // WebSocket
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state((state, engine));

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse().map_err(|e| {
        EngineError::ServerBind { addr: addr_str.clone(), reason: format!("Invalid address: {}", e) }
    })?;
    info!("Dashboard + Command API on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
        EngineError::ServerBind { addr: addr.to_string(), reason: format!("Bind failed: {}", e) }
    })?;
    axum::serve(listener, app).await.map_err(|e| {
        EngineError::ServerBind { addr: addr.to_string(), reason: format!("Server error: {}", e) }
    })?;

    Ok(())
}

// ═══════════════════════════════════════════
// READ ENDPOINTS (GET)
// ═══════════════════════════════════════════

async fn get_state(
    State((state, _)): State<(SharedState, SharedEngine)>,
) -> impl IntoResponse {
    let s = state.read().await;
    Json(serde_json::json!({
        "capital": s.portfolio.capital,
        "initial_capital": s.portfolio.initial_capital,
        "total_pnl": s.portfolio.total_pnl,
        "total_pnl_pct": s.portfolio.total_pnl_pct,
        "daily_pnl": s.portfolio.daily_pnl,
        "agent_state": s.portfolio.agent_state,
        "hunger_level": s.portfolio.hunger_level,
        "win_rate": s.portfolio.win_rate,
        "total_trades": s.portfolio.total_trades,
        "winning_trades": s.portfolio.winning_trades,
        "position_count": s.portfolio.positions.len(),
        "market_count": s.cached_markets.len(),
        "engine_running": s.engine_running,
        "last_scan": s.last_scan,
        "last_command": s.last_command,
        "last_trade_at": s.portfolio.last_trade_at,
        "last_profit_at": s.portfolio.last_profit_at,
        "uptime_hours": s.portfolio.uptime_hours,
        "timestamp": s.portfolio.timestamp,
        "api_budget_used_pct": s.api_costs.usage_pct(),
        "api_daily_cost": s.api_costs.daily_cost_usd,
        "api_total_cost": s.api_costs.total_cost_usd,
        "errors": s.errors.iter().rev().take(10).collect::<Vec<_>>(),
    }))
}

async fn get_positions(
    State((state, _)): State<(SharedState, SharedEngine)>,
) -> impl IntoResponse {
    let s = state.read().await;
    Json(&s.portfolio.positions)
}

async fn get_trades(
    State((state, _)): State<(SharedState, SharedEngine)>,
) -> impl IntoResponse {
    let s = state.read().await;
    Json(&s.portfolio.recent_trades)
}

async fn get_markets(
    State((state, _)): State<(SharedState, SharedEngine)>,
) -> impl IntoResponse {
    let s = state.read().await;
    let markets: Vec<serde_json::Value> = s
        .cached_markets
        .iter()
        .take(50)
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "question": m.question,
                "yes_price": m.yes_price(),
                "no_price": m.no_price(),
                "volume_24hr": m.volume_24hr,
                "liquidity": m.liquidity,
                "best_bid": m.best_bid,
                "best_ask": m.best_ask,
                "category": m.category,
                "end_date": m.end_date,
                "spread": m.spread(),
            })
        })
        .collect();
    Json(markets)
}

async fn get_costs(
    State((state, _)): State<(SharedState, SharedEngine)>,
) -> impl IntoResponse {
    let s = state.read().await;
    Json(serde_json::json!({
        "total_input_tokens": s.api_costs.total_input_tokens,
        "total_output_tokens": s.api_costs.total_output_tokens,
        "total_cache_read_tokens": s.api_costs.total_cache_read_tokens,
        "total_cache_write_tokens": s.api_costs.total_cache_write_tokens,
        "total_cost_usd": s.api_costs.total_cost_usd,
        "daily_cost_usd": s.api_costs.daily_cost_usd,
        "daily_budget": s.api_costs.daily_budget,
        "budget_remaining": s.api_costs.budget_remaining(),
        "calls_today": s.api_costs.calls_today,
        "usage_pct": s.api_costs.usage_pct(),
    }))
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "engine": "polymarket-engine",
        "mode": "command-driven",
        "version": "0.2.0"
    }))
}

// ═══════════════════════════════════════════
// COMMAND ENDPOINTS (POST) — OpenClaw → Engine
// ═══════════════════════════════════════════

/// POST /api/trade — OpenClaw tells engine to execute a trade
async fn post_trade(
    State((_, engine)): State<(SharedState, SharedEngine)>,
    Json(cmd): Json<TradeCommand>,
) -> Result<Json<CommandResponse>, EngineError> {
    let resp = engine.handle_trade_command(cmd).await?;
    Ok(Json(resp))
}

/// POST /api/close — OpenClaw tells engine to close a position
async fn post_close(
    State((_, engine)): State<(SharedState, SharedEngine)>,
    Json(cmd): Json<CloseCommand>,
) -> Result<Json<CommandResponse>, EngineError> {
    let resp = engine.handle_close_command(cmd).await?;
    Ok(Json(resp))
}

/// POST /api/report-cost — OpenClaw reports its Claude API usage for tracking
async fn post_report_cost(
    State((_, engine)): State<(SharedState, SharedEngine)>,
    Json(report): Json<ApiCostReport>,
) -> Result<Json<serde_json::Value>, EngineError> {
    engine.handle_cost_report(report).await;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ═══════════════════════════════════════════
// WEBSOCKET (real-time updates for dashboard)
// ═══════════════════════════════════════════

async fn ws_handler(
    ws: WebSocketUpgrade,
    State((state, _)): State<(SharedState, SharedEngine)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    info!("Dashboard WebSocket client connected");

    let send_state = state.clone();
    let mut send_task = tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2));
        let mut consecutive_errors: u32 = 0;
        loop {
            interval.tick().await;
            let s = send_state.read().await;
            let msg = WsMessage::state_update(&s.portfolio);
            drop(s);
            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(e) => {
                    warn!("WS serialization error: {}", e);
                    continue;
                }
            };
            if sender.send(Message::Text(json.into())).await.is_err() {
                consecutive_errors += 1;
                if consecutive_errors >= 3 {
                    warn!("WS send failed {} times, closing", consecutive_errors);
                    break;
                }
            } else {
                consecutive_errors = 0;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    info!("WS command: {}", text);
                }
                Ok(Message::Close(_)) => {
                    info!("WS client sent close frame");
                    break;
                }
                Err(e) => {
                    warn!("WS receive error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    info!("Dashboard WebSocket client disconnected");
}
