use axum::{
    Json, Router,
    routing::{get, post},
};
use num_bigint::BigInt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use uuid::Uuid;

static CALCULATION_ACTIVE: AtomicBool = AtomicBool::new(false);

struct CalculationGuard;

impl CalculationGuard {
    fn new() -> Self {
        CALCULATION_ACTIVE.store(true, Ordering::SeqCst);
        CalculationGuard
    }
}

impl Drop for CalculationGuard {
    fn drop(&mut self) {
        CALCULATION_ACTIVE.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Serialize)]
struct Skill {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    tags: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct AgentCard {
    name: &'static str,
    description: String,
    #[serde(rename = "protocolVersion")]
    protocol_version: &'static str,
    version: &'static str,
    url: String,
    skills: Vec<Skill>,
    capabilities: serde_json::Value,
    #[serde(rename = "defaultInputModes")]
    default_input_modes: Vec<&'static str>,
    #[serde(rename = "defaultOutputModes")]
    default_output_modes: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: serde_json::Value,
    method: String,
    params: Option<MessageSendParams>,
}

#[derive(Debug, Deserialize)]
struct MessageSendParams {
    message: Message,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Message {
    kind: String,
    #[serde(rename = "messageId")]
    message_id: String,
    role: String,
    parts: Vec<Part>,
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    context_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "kind")]
enum Part {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: serde_json::Value,
    result: Message,
}

#[derive(Debug, Serialize)]
struct JsonRpcErrorResponse {
    jsonrpc: String,
    id: serde_json::Value,
    error: JsonRpcError,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[allow(clippy::manual_is_multiple_of)]
fn is_prime(n: u64) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

fn is_mersenne_prime(p: u64) -> bool {
    if p == 2 {
        return true;
    }
    let m_p = (BigInt::from(1) << p as usize) - 1;
    let mut s = BigInt::from(4);
    let two = BigInt::from(2);
    for _ in 0..(p - 2) {
        s = &s * &s - &two;
        while s > m_p {
            let a = &s >> p as usize;
            let b = &s & &m_p;
            s = a + b;
        }
        if s == m_p {
            s = BigInt::from(0);
        }
    }
    s == BigInt::from(0)
}

fn find_mersenne_primes(count: usize) -> (Vec<BigInt>, f64) {
    let start = Instant::now();
    let mut primes = Vec::new();
    let exponents = [
        2, 3, 5, 7, 13, 17, 19, 31, 61, 89, 107, 127, 521, 607, 1279, 2203, 2281, 3217, 4253, 4423,
        9689, 9941, 11213, 19937, 21701, 23209,
    ];
    for i in 0..count {
        if i >= exponents.len() {
            break;
        }
        let p = exponents[i];
        if is_mersenne_prime(p) {
            let m_p = (BigInt::from(1) << p as usize) - 1;
            primes.push(m_p);
        }
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (primes, elapsed)
}

async fn get_agent_card() -> Json<AgentCard> {
    let model_name = std::env::var("MODEL_NAME").unwrap_or_else(|_| "Not specified".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8104".to_string());
    let public_url =
        std::env::var("PUBLIC_URL").unwrap_or_else(|_| format!("http://0.0.0.0:{}", port));
    Json(AgentCard {
        name: "Mersenne Prime Agent Rust",
        description: format!(
            "A rust agent that builds a list of the first n Mersenne primes and reports the elapsed time. Configured with model: {}.",
            model_name
        ),
        protocol_version: "0.3.0",
        version: "0.1.0",
        url: public_url,
        skills: vec![
            Skill {
                id: "find-mersenne-rust",
                name: "Find Mersenne Primes in rust",
                description: "Finds the list of the first n Mersenne primes in Rust, or calculates a specific Mersenne prime for a given exponent parameter.",
                tags: vec!["math", "benchmark"],
            },
            Skill {
                id: "check-status-rust",
                name: "Check calculation status in Rust",
                description: "Returns 'ready' if no calculation is active, and 'not ready' if a calculation is live.",
                tags: vec!["status", "health"],
            },
        ],
        capabilities: serde_json::json!({}),
        default_input_modes: vec![],
        default_output_modes: vec![],
    })
}

async fn handle_rpc(Json(req): Json<JsonRpcRequest>) -> Json<serde_json::Value> {
    if req.method != "message/send" {
        return Json(
            serde_json::to_value(JsonRpcErrorResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                error: JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", req.method),
                },
            })
            .unwrap(),
        );
    }

    let params = match req.params {
        Some(p) => p,
        None => {
            return Json(
                serde_json::to_value(JsonRpcErrorResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    error: JsonRpcError {
                        code: -32602,
                        message: "Invalid method parameters: params missing".to_string(),
                    },
                })
                .unwrap(),
            );
        }
    };

    static RE_STATUS: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re_status = RE_STATUS.get_or_init(|| Regex::new(r"(?i)\b(status|ready|active)\b").unwrap());

    let mut is_status_query = false;
    for part in &params.message.parts {
        match part {
            Part::Text { text } => {
                if re_status.is_match(text) {
                    is_status_query = true;
                    break;
                }
            }
        }
    }

    if is_status_query {
        let is_active = CALCULATION_ACTIVE.load(Ordering::SeqCst);
        let status_msg = if is_active { "not ready" } else { "ready" };
        let response_message = Message {
            kind: "message".to_string(),
            message_id: Uuid::new_v4().to_string(),
            role: "agent".to_string(),
            parts: vec![Part::Text {
                text: status_msg.to_string(),
            }],
            context_id: params.message.context_id,
        };

        return Json(
            serde_json::to_value(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: response_message,
            })
            .unwrap(),
        );
    }

    static RE_EXPONENT: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re_exponent = RE_EXPONENT
        .get_or_init(|| Regex::new(r"(?i)\b(exponent|exp|p)\b\s*[:=]?\s*(\d+)").unwrap());

    static RE_NUMBER: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re_number = RE_NUMBER.get_or_init(|| Regex::new(r"(\d+)").unwrap());

    let mut exponent: Option<u64> = None;

    for part in &params.message.parts {
        match part {
            Part::Text { text } => {
                if let Some(caps) = re_exponent.captures(text)
                    && let Some(m) = caps.get(2)
                    && let Ok(parsed) = m.as_str().parse::<u64>()
                {
                    exponent = Some(parsed);
                    break;
                } else if let Some(caps) = re_number.captures(text)
                    && let Some(m) = caps.get(1)
                    && let Ok(parsed) = m.as_str().parse::<u64>()
                {
                    exponent = Some(parsed);
                    break;
                }
            }
        }
    }

    let context_id = params.message.context_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        let _guard = CalculationGuard::new();

        if let Some(p) = exponent {
            if !is_prime(p) {
                return "not prime".to_string();
            }
            if is_mersenne_prime(p) {
                let m_p: BigInt = (BigInt::from(1) << p as usize) - 1;
                m_p.to_string()
            } else {
                "not prime".to_string()
            }
        } else {
            let c = 5;
            let (primes, elapsed) = find_mersenne_primes(c);

            format!(
                "Found first {} Mersenne primes in {:.2}ms. Primes: {:?}",
                primes.len(),
                elapsed,
                primes
            )
        }
    })
    .await;

    let response_text = match result {
        Ok(text) => text,
        Err(e) => format!("Calculation panicked or failed: {}", e),
    };

    let response_message = Message {
        kind: "message".to_string(),
        message_id: Uuid::new_v4().to_string(),
        role: "agent".to_string(),
        parts: vec![Part::Text {
            text: response_text,
        }],
        context_id,
    };

    Json(
        serde_json::to_value(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: response_message,
        })
        .unwrap(),
    )
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    let model_name = std::env::var("MODEL_NAME").unwrap_or_else(|_| "Not specified".to_string());
    println!("Agent configured with MODEL_NAME: {}", model_name);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/.well-known/agent-card.json", get(get_agent_card))
        .route("/.well-known/agent.json", get(get_agent_card))
        .route("/", post(handle_rpc));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8104".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 Server started on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
