use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

// ═══════════════════════════════════════════
// ENGINE ERROR TYPES
// ═══════════════════════════════════════════

#[derive(Debug, Error)]
pub enum EngineError {
    // --- Config ---
    #[error("Missing required env var: {0}")]
    MissingConfig(String),

    #[error("Invalid config value for {key}: {reason}")]
    InvalidConfig { key: String, reason: String },

    // --- Polymarket API ---
    #[error("Polymarket API error ({status}): {message}")]
    PolymarketApi { status: u16, message: String },

    #[error("Polymarket request timeout after {0}s")]
    PolymarketTimeout(u64),

    #[error("Polymarket rate limited, retry after {retry_after_ms}ms")]
    PolymarketRateLimit { retry_after_ms: u64 },

    #[error("Failed to parse Polymarket response: {0}")]
    PolymarketParse(String),

    // --- Trade Execution ---
    #[error("Market {0} not found in cache")]
    MarketNotFound(String),

    #[error("No token IDs for market {0}")]
    NoTokenIds(String),

    #[error("Position {0} not found")]
    PositionNotFound(String),

    #[error("Position size {size:.2} exceeds limit {limit:.2}")]
    PositionSizeExceeded { size: f64, limit: f64 },

    #[error("Cost ${cost:.2} exceeds {pct:.0}% of capital ${capital:.2}")]
    InsufficientCapital { cost: f64, capital: f64, pct: f64 },

    #[error("Order rejected by CLOB: {0}")]
    OrderRejected(String),

    #[error("Max positions ({0}) reached")]
    MaxPositionsReached(usize),

    #[error("Market not active or order book disabled")]
    MarketNotTradeable,

    // --- Risk ---
    #[error("Edge {edge:.1}% below minimum {min:.1}% for state {state}")]
    EdgeTooLow { edge: f64, min: f64, state: String },

    #[error("Confidence {0:.1}% below minimum threshold")]
    LowConfidence(f64),

    #[error("Daily loss limit reached: ${0:.2}")]
    DailyLossLimit(f64),

    // --- Network ---
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Connection failed to {url}: {reason}")]
    ConnectionFailed { url: String, reason: String },

    // --- Serialization ---
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    // --- Server ---
    #[error("Server bind failed on {addr}: {reason}")]
    ServerBind { addr: String, reason: String },

    #[error("Invalid request: {0}")]
    BadRequest(String),

    // --- Database ---
    #[error("Database error: {0}")]
    Database(String),

    // --- Internal ---
    #[error("Internal engine error: {0}")]
    Internal(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// ═══════════════════════════════════════════
// ERROR CLASSIFICATION
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ErrorSeverity {
    /// Informational — logged but doesn't affect operation
    Info,
    /// Warning — degraded but still functional
    Warning,
    /// Error — operation failed, will retry
    Retryable,
    /// Critical — needs intervention
    Critical,
    /// Fatal — engine should shut down
    Fatal,
}

impl EngineError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Retryable network issues
            Self::Http(_) | Self::PolymarketTimeout(_) | Self::WebSocket(_) => {
                ErrorSeverity::Retryable
            }

            // Rate limits — back off and retry
            Self::PolymarketRateLimit { .. } => ErrorSeverity::Retryable,

            // Connection failure — retryable
            Self::ConnectionFailed { .. } => ErrorSeverity::Retryable,

            // Business logic rejections — not retryable but not severe
            Self::MarketNotFound(_)
            | Self::PositionNotFound(_)
            | Self::NoTokenIds(_)
            | Self::MarketNotTradeable
            | Self::EdgeTooLow { .. }
            | Self::LowConfidence(_) => ErrorSeverity::Warning,

            // Risk limits — warning
            Self::PositionSizeExceeded { .. }
            | Self::InsufficientCapital { .. }
            | Self::MaxPositionsReached(_)
            | Self::DailyLossLimit(_) => ErrorSeverity::Warning,

            // Order rejected — could be transient
            Self::OrderRejected(_) => ErrorSeverity::Warning,

            // Parse/serialization — probably a bug
            Self::PolymarketParse(_) | Self::Json(_) => ErrorSeverity::Warning,

            // Bad request from OpenClaw
            Self::BadRequest(_) => ErrorSeverity::Info,

            // Config errors — fatal, can't start
            Self::MissingConfig(_) | Self::InvalidConfig { .. } => ErrorSeverity::Fatal,

            // Server bind — fatal
            Self::ServerBind { .. } => ErrorSeverity::Fatal,

            // Database — critical but engine can continue without persistence
            Self::Database(_) => ErrorSeverity::Critical,

            // Internal / other — critical
            Self::Internal(_) | Self::Other(_) => ErrorSeverity::Critical,

            // Polymarket API error — depends on status
            Self::PolymarketApi { status, .. } => {
                if *status == 429 {
                    ErrorSeverity::Retryable
                } else if *status >= 500 {
                    ErrorSeverity::Retryable
                } else {
                    ErrorSeverity::Warning
                }
            }
        }
    }

    /// Whether the operation can be retried
    pub fn is_retryable(&self) -> bool {
        self.severity() == ErrorSeverity::Retryable
    }

    /// Human-readable error code for logging/dashboard
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingConfig(_) => "CONFIG_MISSING",
            Self::InvalidConfig { .. } => "CONFIG_INVALID",
            Self::PolymarketApi { .. } => "POLY_API_ERR",
            Self::PolymarketTimeout(_) => "POLY_TIMEOUT",
            Self::PolymarketRateLimit { .. } => "POLY_RATE_LIMIT",
            Self::PolymarketParse(_) => "POLY_PARSE_ERR",
            Self::MarketNotFound(_) => "MKT_NOT_FOUND",
            Self::NoTokenIds(_) => "NO_TOKEN_IDS",
            Self::PositionNotFound(_) => "POS_NOT_FOUND",
            Self::PositionSizeExceeded { .. } => "SIZE_EXCEEDED",
            Self::InsufficientCapital { .. } => "INSUFF_CAPITAL",
            Self::OrderRejected(_) => "ORDER_REJECTED",
            Self::MaxPositionsReached(_) => "MAX_POSITIONS",
            Self::MarketNotTradeable => "MKT_NOT_TRADEABLE",
            Self::EdgeTooLow { .. } => "EDGE_TOO_LOW",
            Self::LowConfidence(_) => "LOW_CONFIDENCE",
            Self::DailyLossLimit(_) => "DAILY_LOSS_LIMIT",
            Self::Http(_) => "HTTP_ERR",
            Self::WebSocket(_) => "WS_ERR",
            Self::ConnectionFailed { .. } => "CONN_FAILED",
            Self::Json(_) => "JSON_ERR",
            Self::ServerBind { .. } => "SERVER_BIND_ERR",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Database(_) => "DB_ERR",
            Self::Internal(_) => "INTERNAL_ERR",
            Self::Other(_) => "UNKNOWN_ERR",
        }
    }

    /// How long to wait before retrying (exponential backoff hint)
    pub fn retry_delay_ms(&self, attempt: u32) -> u64 {
        let base = match self {
            Self::PolymarketRateLimit { retry_after_ms } => *retry_after_ms,
            Self::PolymarketTimeout(_) => 5_000,
            Self::Http(_) | Self::ConnectionFailed { .. } => 3_000,
            Self::PolymarketApi { status, .. } if *status >= 500 => 5_000,
            _ => 1_000,
        };
        // Exponential backoff: base * 2^attempt, capped at 60s
        (base * 2u64.pow(attempt)).min(60_000)
    }
}

// ═══════════════════════════════════════════
// AXUM ERROR RESPONSE
// HTTP error responses for REST API
// ═══════════════════════════════════════════

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: String,
    pub severity: ErrorSeverity,
    pub retryable: bool,
}

impl IntoResponse for EngineError {
    fn into_response(self) -> Response {
        let status = match &self {
            EngineError::BadRequest(_) => StatusCode::BAD_REQUEST,
            EngineError::MarketNotFound(_) | EngineError::PositionNotFound(_) => {
                StatusCode::NOT_FOUND
            }
            EngineError::PolymarketRateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            EngineError::InsufficientCapital { .. }
            | EngineError::PositionSizeExceeded { .. }
            | EngineError::MaxPositionsReached(_)
            | EngineError::EdgeTooLow { .. }
            | EngineError::LowConfidence(_)
            | EngineError::DailyLossLimit(_)
            | EngineError::MarketNotTradeable
            | EngineError::NoTokenIds(_) => StatusCode::UNPROCESSABLE_ENTITY,
            EngineError::OrderRejected(_) => StatusCode::CONFLICT,
            EngineError::PolymarketTimeout(_)
            | EngineError::Http(_)
            | EngineError::ConnectionFailed { .. } => StatusCode::BAD_GATEWAY,
            EngineError::PolymarketApi { status, .. } => {
                StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ErrorResponse {
            success: false,
            error: self.to_string(),
            code: self.code().to_string(),
            severity: self.severity(),
            retryable: self.is_retryable(),
        };

        (status, Json(body)).into_response()
    }
}

// ═══════════════════════════════════════════
// ERROR LOGGING HELPER
// ═══════════════════════════════════════════

/// Log error with appropriate tracing level based on severity
pub fn log_error(err: &EngineError) {
    match err.severity() {
        ErrorSeverity::Info => tracing::info!("[{}] {}", err.code(), err),
        ErrorSeverity::Warning => tracing::warn!("[{}] {}", err.code(), err),
        ErrorSeverity::Retryable => tracing::warn!("[{}] {} (retryable)", err.code(), err),
        ErrorSeverity::Critical => tracing::error!("[{}] CRITICAL: {}", err.code(), err),
        ErrorSeverity::Fatal => tracing::error!("[{}] FATAL: {}", err.code(), err),
    }
}

pub type EngineResult<T> = Result<T, EngineError>;
