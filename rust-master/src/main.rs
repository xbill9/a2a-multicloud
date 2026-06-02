use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// A2A Models
// ============================================================================

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
    url: &'static str,
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

// ============================================================================
// MCP Models (HTTP transport, request-response JSON-RPC)
// ============================================================================

#[derive(Debug, Deserialize)]
struct McpPostMessage {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

// ============================================================================
// Gemini API Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize)]
struct SystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
}

// ============================================================================
// Authentication & API Helpers
// ============================================================================

/// Retrieves a GCP access token for Vertex AI invocation.
async fn get_gcp_token() -> Option<String> {
    if let Ok(token) = std::env::var("GCP_ACCESS_TOKEN") {
        return Some(token);
    }
    // Try local gcloud command
    if let Ok(output) = std::process::Command::new("gcloud")
        .args(["auth", "print-access-token"])
        .output()
    {
        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Some(token);
        }
    }
    // Try metadata server (GCP environment e.g. Cloud Run)
    let client = reqwest::Client::new();
    if let Ok(resp) = client
        .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
        .header("Metadata-Flavor", "Google")
        .send()
        .await
    {
        #[derive(Deserialize)]
        struct TokenResp {
            access_token: String,
        }
        if let Ok(token_resp) = resp.json::<TokenResp>().await {
            return Some(token_resp.access_token);
        }
    }
    None
}

/// Retrieves a GCP identity (OIDC ID) token for Cloud Run target invocations.
async fn get_gcp_id_token() -> Option<String> {
    if let Ok(token) = std::env::var("GCP_ID_TOKEN") {
        return Some(token);
    }
    // Try local gcloud command
    if let Ok(output) = std::process::Command::new("gcloud")
        .args(["auth", "print-identity-token"])
        .output()
    {
        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Some(token);
        }
    }
    None
}

/// Retrieves GCP project ID.
fn get_gcp_project() -> Option<String> {
    if let Ok(proj) = std::env::var("GOOGLE_CLOUD_PROJECT") {
        return Some(proj);
    }
    // Try standard project config file or system variables
    if let Ok(output) = std::process::Command::new("gcloud")
        .args(["config", "get-value", "project"])
        .output()
    {
        if output.status.success() {
            let proj = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !proj.is_empty() && proj != "(unset)" {
                return Some(proj);
            }
        }
    }
    None
}

/// Calls a sub-agent using the A2A POST JSON-RPC protocol.
async fn call_sub_agent(agent_name: &str, query: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url =
        match agent_name {
            "rust_agent" => std::env::var("RUST_AGENT_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8104".to_string()),
            "gcp_agent" => std::env::var("GCP_AGENT_URL").unwrap_or_else(|_| {
                "https://bench-rust-wgcq55zbfq-uc.a.run.app".to_string()
            }),
            "aws_agent" => std::env::var("AWS_AGENT_URL").unwrap_or_else(|_| {
                "https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com".to_string()
            }),
            "azure_agent" => std::env::var("AZURE_AGENT_URL").unwrap_or_else(|_| {
                "https://a2a-app-penguin.icyplant-a768d75c.westus2.azurecontainerapps.io".to_string()
            }),
            _ => return Err(format!("Unknown agent name: {}", agent_name)),
        };

    tracing::info!("Calling sub-agent: {} at URL: {}", agent_name, url);

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "message/send",
        "params": {
            "message": {
                "kind": "message",
                "messageId": Uuid::new_v4().to_string(),
                "role": "user",
                "parts": [
                    {
                        "kind": "text",
                        "text": query
                    }
                ],
                "contextId": Uuid::new_v4().to_string()
            }
        }
    });

    let mut req = client.post(&url).json(&payload);
    if agent_name == "gcp_agent" {
        if let Some(token) = get_gcp_id_token().await {
            req = req.bearer_auth(token);
        }
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request to sub-agent failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Sub-agent returned HTTP status {}", resp.status()));
    }

    let json_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse sub-agent response JSON: {}", e))?;

    if let Some(error) = json_resp.get("error") {
        let err_msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("Sub-agent returned error: {}", err_msg));
    }

    let text_val = json_resp
        .pointer("/result/parts/0/text")
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            format!(
                "Could not find result text in sub-agent response structure: {:?}",
                json_resp
            )
        })?;

    Ok(text_val.to_string())
}

#[derive(Debug, Serialize)]
struct BenchmarkRun {
    timestamp: String,
    n: i64,
    #[serde(rename = "cumulativeDurationMs")]
    cumulative_duration_ms: f64,
    agents: Vec<AgentBenchmark>,
}

#[derive(Debug, Serialize)]
struct AgentBenchmark {
    #[serde(rename = "agentId")]
    agent_id: String,
    #[serde(rename = "agentName")]
    agent_name: String,
    #[serde(rename = "totalDurationMs")]
    total_duration_ms: f64,
    #[serde(rename = "readyChecksPassed")]
    ready_checks_passed: usize,
    #[serde(rename = "calcCallsSuccessful")]
    calc_calls_successful: usize,
    calls: Vec<CallTiming>,
}

#[derive(Debug, Serialize)]
struct CallTiming {
    exponent: i64,
    #[serde(rename = "statusCheckDurationMs")]
    status_check_duration_ms: f64,
    #[serde(rename = "statusResult")]
    status_result: String,
    #[serde(rename = "calculationDurationMs")]
    calculation_duration_ms: f64,
    #[serde(rename = "calculationResult")]
    calculation_result: String,
}

/// Runs a benchmark to check readiness and then send calculation queries to the agents.
async fn run_a2a_benchmark(n: i64) -> Result<String, String> {
    let timestamp = chrono::Local::now().to_rfc3339();
    let file_timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    
    let mut results = Vec::new();
    let agents = vec![
        ("gcp_agent", "GCP (Cloud Run)"),
        ("aws_agent", "AWS (Lightsail)"),
        ("azure_agent", "Azure (Container Apps)"),
        ("rust_agent", "Rust (Local)"),
    ];

    results.push(format!("### Distributed Mersenne Prime Generation from 1 to {}\n", n));

    let mut summary_table = String::new();
    summary_table.push_str("| Exponent | Assigned Agent | Ready Check Time (ms) | Calc Time (ms) | Result (Mersenne Prime) |\n");
    summary_table.push_str("|---|---|---|---|---|\n");

    let total_start = std::time::Instant::now();
    let mut coordinated_calls = Vec::new();
    let mut generated_primes = Vec::new();

    let mut current_agent_idx = 0;

    for i in 1..=n {
        let mut assigned = false;
        let mut attempts = 0;
        let max_attempts = agents.len() * 2;
        
        let mut assigned_agent_name = "None";
        let mut status_elapsed = 0.0;
        let mut status_msg = "failed".to_string();
        let mut calc_elapsed = 0.0;
        let mut calc_msg = "Failed assignment".to_string();

        while !assigned && attempts < max_attempts {
            let (agent_id, agent_name) = agents[current_agent_idx];
            current_agent_idx = (current_agent_idx + 1) % agents.len();
            attempts += 1;

            // Check if agent is ready
            let status_start = std::time::Instant::now();
            let status_res = call_sub_agent(agent_id, "status").await;
            status_elapsed = status_start.elapsed().as_secs_f64() * 1000.0;

            match status_res {
                Ok(ref s) if s.trim() == "ready" => {
                    status_msg = "ready".to_string();
                    assigned_agent_name = agent_name;
                    
                    // Send calculation
                    tracing::info!("Coordinating: Exponent {} is assigned to '{}' ({})", i, agent_name, agent_id);
                    let calc_start = std::time::Instant::now();
                    let calc_res = call_sub_agent(agent_id, &format!("{}", i)).await;
                    calc_elapsed = calc_start.elapsed().as_secs_f64() * 1000.0;
                    
                    match calc_res {
                        Ok(res) => {
                            calc_msg = res.trim().to_string();
                            if calc_msg != "not prime" && !calc_msg.is_empty() {
                                generated_primes.push(format!("- 2^{} - 1 = {} (via {})", i, calc_msg, agent_name));
                            }
                            tracing::info!("Coordinating: Exponent {} completed by '{}' in {:.2}ms with result: {}", i, agent_name, calc_elapsed, calc_msg);
                        }
                        Err(e) => {
                            calc_msg = format!("Error: {}", e);
                            tracing::error!("Coordinating: Exponent {} call to '{}' failed: {}", i, agent_name, e);
                        }
                    }
                    assigned = true;
                }
                Ok(s) => {
                    status_msg = s;
                }
                Err(e) => {
                    status_msg = format!("Error: {}", e);
                }
            }
        }

        if !assigned {
            calc_msg = "Failed (No active/ready agents)".to_string();
            assigned_agent_name = "None (All busy/offline)";
        }

        summary_table.push_str(&format!(
            "| {} | {} | {:.2} ms | {:.2} ms | {} |\n",
            i, assigned_agent_name, status_elapsed, calc_elapsed, calc_msg
        ));

        coordinated_calls.push(CallTiming {
            exponent: i,
            status_check_duration_ms: status_elapsed,
            status_result: status_msg,
            calculation_duration_ms: calc_elapsed,
            calculation_result: calc_msg,
        });
    }

    let cumulative_time = total_start.elapsed().as_secs_f64() * 1000.0;
    
    // Save to disk
    let run_data = BenchmarkRun {
        timestamp: timestamp.clone(),
        n,
        cumulative_duration_ms: cumulative_time,
        agents: vec![
            AgentBenchmark {
                agent_id: "coordinator".to_string(),
                agent_name: "Master Coordinator".to_string(),
                total_duration_ms: cumulative_time,
                ready_checks_passed: coordinated_calls.iter().filter(|c| c.status_result == "ready").count(),
                calc_calls_successful: coordinated_calls.iter().filter(|c| !c.calculation_result.starts_with("Error") && !c.calculation_result.starts_with("Failed")).count(),
                calls: coordinated_calls,
            }
        ],
    };

    if let Ok(json_str) = serde_json::to_string_pretty(&run_data) {
        let filename = format!("benchmark_results_{}.json", file_timestamp);
        let _ = std::fs::write(&filename, &json_str);
        let _ = std::fs::write("benchmark_results.json", &json_str);
    }

    let mut final_report = String::new();
    final_report.push_str("## Distributed Mersenne Prime Generation Report\n\n");
    final_report.push_str(&format!("- **Timestamp:** {}\n", timestamp));
    final_report.push_str(&format!("- **Cumulative Duration:** {:.2} ms\n\n", cumulative_time));
    
    final_report.push_str("### Generated Mersenne Primes:\n");
    if generated_primes.is_empty() {
        final_report.push_str("None found in this range.\n");
    } else {
        final_report.push_str(&generated_primes.join("\n"));
        final_report.push_str("\n");
    }
    final_report.push_str("\n### Coordinated Exponent Assignments\n\n");
    final_report.push_str(&summary_table);
    
    Ok(final_report)
}

fn extract_number_from_query(query: &str) -> Option<i64> {
    let mut start_idx = None;
    let mut end_idx = None;
    for (i, c) in query.char_indices() {
        if c.is_ascii_digit() {
            if start_idx.is_none() {
                start_idx = Some(i);
            }
            end_idx = Some(i + c.len_utf8());
        } else if start_idx.is_some() {
            break;
        }
    }
    
    if let (Some(start), Some(end)) = (start_idx, end_idx) {
        query[start..end].parse::<i64>().ok()
    } else {
        None
    }
}

/// Coordinates task execution by using a Gemini LLM loop with tools mapping to A2A sub-agents.
async fn run_coordinator(query: &str) -> Result<String, String> {
    let model_name = std::env::var("MODEL_NAME").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    let use_vertex =
        std::env::var("GOOGLE_GENAI_USE_VERTEXAI").unwrap_or_else(|_| "0".to_string()) == "1";

    let mut contents = vec![GeminiContent {
        role: "user".to_string(),
        parts: vec![GeminiPart {
            text: Some(query.to_string()),
            function_call: None,
            function_response: None,
        }],
    }];

    let client = reqwest::Client::new();
    let max_iterations = 5;

    for i in 0..max_iterations {
        tracing::info!("LLM Coordinator loop iteration {}", i + 1);

        let system_instruction = SystemInstruction {
            parts: vec![GeminiPart {
                text: Some(
                    "You are the Master A2A Agent\n\
                      you delegate to your sub agents by the a2a protocol\n\
                      if the user asks about calculating Mersenne primes or checking exponents, use rust_agent, gcp_agent, aws_agent, or azure_agent to calculate intermediate numbers or results\n"
                        .to_string(),
                ),
                function_call: None,
                function_response: None,
            }],
        };

        let tools = vec![serde_json::json!({
            "functionDeclarations": [
                {
                    "name": "rust_agent",
                    "description": "Rust Prime Agent. Call this to calculate Mersenne primes or check exponents in Rust.",
                    "parameters": {
                        "type": "OBJECT",
                        "properties": {
                            "query": {
                                "type": "STRING",
                                "description": "The query or instruction to pass to the Rust agent."
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "gcp_agent",
                    "description": "GCP Prime Agent. Call this to calculate Mersenne primes or check exponents on GCP (Cloud Run).",
                    "parameters": {
                        "type": "OBJECT",
                        "properties": {
                            "query": {
                                "type": "STRING",
                                "description": "The query or instruction to pass to the GCP agent."
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "aws_agent",
                    "description": "AWS Prime Agent. Call this to calculate Mersenne primes or check exponents on AWS (Lightsail).",
                    "parameters": {
                        "type": "OBJECT",
                        "properties": {
                            "query": {
                                "type": "STRING",
                                "description": "The query or instruction to pass to the AWS agent."
                            }
                        },
                        "required": ["query"]
                    }
                },
                {
                    "name": "azure_agent",
                    "description": "Azure Prime Agent. Call this to calculate Mersenne primes or check exponents on Azure (Container Apps).",
                    "parameters": {
                        "type": "OBJECT",
                        "properties": {
                            "query": {
                                "type": "STRING",
                                "description": "The query or instruction to pass to the Azure agent."
                            }
                        },
                        "required": ["query"]
                    }
                }
            ]
        })];

        let req_body = GeminiRequest {
            contents: contents.clone(),
            system_instruction: Some(system_instruction),
            tools: Some(tools),
        };

        let request_builder = if use_vertex {
            let location = std::env::var("GOOGLE_CLOUD_LOCATION")
                .unwrap_or_else(|_| "us-central1".to_string());
            let project = get_gcp_project().ok_or_else(|| {
                "GCP Project ID not found. Set GOOGLE_CLOUD_PROJECT or verify gcloud setup."
                    .to_string()
            })?;
            let token = get_gcp_token().await.ok_or_else(|| {
                "GCP access token not found. Verify gcloud credentials.".to_string()
            })?;

            let url = format!(
                "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
                location, project, location, model_name
            );
            tracing::info!("Sending request to Vertex AI endpoint: {}", url);
            client.post(&url).bearer_auth(token)
        } else {
            let api_key = std::env::var("GEMINI_API_KEY")
                .map_err(|_| "GEMINI_API_KEY environment variable is not set. Set GEMINI_API_KEY or configure Vertex AI.".to_string())?;
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, api_key
            );
            tracing::info!(
                "Sending request to Google AI Developer endpoint for model: {}",
                model_name
            );
            client.post(&url)
        };

        let response = request_builder
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("Request to Gemini API failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            return Err(format!(
                "Gemini API returned status {}: {}",
                status, err_text
            ));
        }

        let gemini_resp: GeminiResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Gemini API response: {}", e))?;

        let candidate = gemini_resp
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .ok_or_else(|| "No candidates found in Gemini response.".to_string())?;

        let candidate_content = candidate
            .content
            .clone()
            .ok_or_else(|| "Candidate content is empty.".to_string())?;

        // Check if candidate_content has a function call
        let function_call = candidate_content
            .parts
            .iter()
            .find_map(|p| p.function_call.clone());

        if let Some(call) = function_call {
            // Find query argument in the args JSON
            let sub_query = call
                .args
                .get("query")
                .and_then(|q| q.as_str())
                .unwrap_or(query);

            tracing::info!(
                "Model invoked function call: {} with query: {}",
                call.name,
                sub_query
            );

            // Execute sub-agent call
            let sub_resp = match call_sub_agent(&call.name, sub_query).await {
                Ok(resp) => resp,
                Err(err) => {
                    tracing::error!("Sub-agent call error: {}", err);
                    format!("Error calling sub-agent {}: {}", call.name, err)
                }
            };

            // Push the assistant's turn and then the tool output
            contents.push(candidate_content);
            contents.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: None,
                    function_call: None,
                    function_response: Some(GeminiFunctionResponse {
                        name: call.name,
                        response: serde_json::json!({
                            "content": sub_resp
                        }),
                    }),
                }],
            });
        } else {
            // No function call, we have a text response!
            let text_part = candidate_content
                .parts
                .iter()
                .find_map(|p| p.text.clone())
                .ok_or_else(|| "No text response found in candidate content.".to_string())?;

            return Ok(text_part);
        }
    }

    Err(
        "Reached maximum agent coordinator loop limit without getting a final text response."
            .to_string(),
    )
}

async fn check_sub_agents_health() -> String {
    let client = reqwest::Client::new();
    let agents = vec![
        ("AWS (Remote)", std::env::var("AWS_AGENT_URL").unwrap_or_else(|_| "https://a2a-lightsail-rust-aws.6wpv8vensby5c.us-east-1.cs.amazonlightsail.com".to_string()), "/health"),
        ("GCP (Remote)", std::env::var("GCP_AGENT_URL").unwrap_or_else(|_| "https://bench-rust-289270257791.us-central1.run.app".to_string()), "/.well-known/agent.json"),
        ("Azure (Remote)", std::env::var("AZURE_AGENT_URL").unwrap_or_else(|_| "https://a2a-app-penguin.icyplant-a768d75c.westus2.azurecontainerapps.io".to_string()), "/.well-known/agent.json"),
        ("Rust (Local)", std::env::var("RUST_AGENT_URL").unwrap_or_else(|_| "http://127.0.0.1:8104".to_string()), "/.well-known/agent.json"),
    ];

    let mut report = String::new();
    report.push_str("Agent Health Check Results:\n");

    for (name, url, path) in agents {
        let full_url = format!("{}{}", url.trim_end_matches('/'), path);
        let mut req = client.get(&full_url);
        
        if name.contains("GCP") {
            if let Some(token) = get_gcp_id_token().await {
                req = req.bearer_auth(token);
            }
        }

        match tokio::time::timeout(std::time::Duration::from_secs(5), req.send()).await {
            Ok(Ok(resp)) => {
                if resp.status().is_success() {
                    report.push_str(&format!("  ✅ {}: ONLINE (HTTP {})\n", name, resp.status().as_u16()));
                } else {
                    report.push_str(&format!("  ❌ {}: OFFLINE (HTTP {})\n", name, resp.status().as_u16()));
                }
            }
            Ok(Err(e)) => {
                report.push_str(&format!("  ❌ {}: OFFLINE (Error: {})\n", name, e));
            }
            Err(_) => {
                report.push_str(&format!("  ❌ {}: OFFLINE (Timeout)\n", name));
            }
        }
    }
    report
}

// ============================================================================
// Endpoint Handlers
// ============================================================================

/// Axum A2A agent card handler.
async fn get_agent_card() -> Json<AgentCard> {
    let model_name = std::env::var("MODEL_NAME").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    Json(AgentCard {
        name: "master_agent",
        description: format!(
            "Master A2A Agent that coordinates prime agents (including rust_agent, gcp_agent, aws_agent, and azure_agent) using model: {}.",
            model_name
        ),
        protocol_version: "0.3.0",
        version: "0.1.0",
        url: "http://0.0.0.0:8100/",
        skills: vec![
            Skill {
                id: "coordinate-agents",
                name: "Coordinate sub-agents",
                description: "Delegates requests to appropriate sub-agents for weather, time, events, prime calculations, and general greetings.",
                tags: vec!["coordinator", "a2a"],
            },
            Skill {
                id: "calculate-mersenne-prime",
                name: "Calculate Mersenne Prime",
                description: "Calculates Mersenne prime of n using connected sub-agents.",
                tags: vec!["mersenne", "prime", "a2a"],
            }
        ],
        capabilities: serde_json::json!({}),
        default_input_modes: vec![],
        default_output_modes: vec![],
    })
}

/// Axum A2A message handler (POST /)
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

    // Extract prompt from user message parts
    let mut query = String::new();
    if let Some(Part::Text { text }) = params.message.parts.first() {
        query = text.clone();
    }

    if query.is_empty() {
        return Json(
            serde_json::to_value(JsonRpcErrorResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                error: JsonRpcError {
                    code: -32602,
                    message: "User message has empty or missing text parts".to_string(),
                },
            })
            .unwrap(),
        );
    }

    tracing::info!("Received A2A request: {}", query);

    let lower_query = query.to_lowercase();
    let has_keyword = lower_query.contains("benchmark")
        || lower_query.contains("calculate")
        || lower_query.contains("check")
        || lower_query.contains("mersenne")
        || lower_query.contains("prime");

    let result_text = if has_keyword {
        if let Some(n) = extract_number_from_query(&query) {
            if n > 0 && n <= 50 {
                tracing::info!("Query contains keyword and number {}. Running direct benchmark...", n);
                match run_a2a_benchmark(n).await {
                    Ok(res) => res,
                    Err(err) => format!("Error running benchmark: {}", err),
                }
            } else {
                format!("Requested benchmark parameter {} is out of bounds (must be between 1 and 50).", n)
            }
        } else {
            match run_coordinator(&query).await {
                Ok(res) => res,
                Err(err) => format!("Error in Master Agent: {}", err),
            }
        }
    } else {
        match run_coordinator(&query).await {
            Ok(res) => res,
            Err(err) => format!("Error in Master Agent: {}", err),
        }
    };

    let response_message = Message {
        kind: "message".to_string(),
        message_id: Uuid::new_v4().to_string(),
        role: "agent".to_string(),
        parts: vec![Part::Text { text: result_text }],
        context_id: params.message.context_id,
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

/// Unified MCP request processor helper.
async fn process_mcp_message_raw(payload: McpPostMessage) -> Option<serde_json::Value> {
    let id = payload.id.clone().unwrap_or(serde_json::Value::Null);

    match payload.method.as_str() {
        "initialize" => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "rust-master",
                    "version": "0.1.0"
                }
            }
        })),
        "notifications/initialized" => None,
        "tools/list" => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "ask_master_agent",
                        "description": "Ask the master agent a question.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "The query to ask the master agent"
                                }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "calculate_mersenne_prime",
                        "description": "Calculate the Mersenne prime of n. Delegates intermediate numbers calculation to connected A2A sub-agents.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "n": {
                                    "type": "integer",
                                    "description": "The exponent (value of n) to check or calculate the Mersenne prime for."
                                }
                            },
                            "required": ["n"]
                        }
                    },
                    {
                        "name": "check_agents_status",
                        "description": "Check the status and health of all connected sub-agents (AWS, GCP, Azure, and local/A2A sub-agents).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    }
                ]
            }
        })),
        "tools/call" => {
            let tool_name = payload
                .params
                .as_ref()
                .and_then(|p| p.get("name").and_then(|n| n.as_str()))
                .unwrap_or("");

            if tool_name == "ask_master_agent" {
                let tool_query = payload
                    .params
                    .as_ref()
                    .and_then(|p| {
                        p.get("arguments")
                            .and_then(|a| a.get("query").and_then(|q| q.as_str()))
                    })
                    .unwrap_or("")
                    .to_string();

                tracing::info!(
                    "MCP tool ask_master_agent called with query: {}",
                    tool_query
                );
                let result_str = match run_coordinator(&tool_query).await {
                    Ok(res) => res,
                    Err(err) => format!("Error in Master Coordinator Loop: {}", err),
                };

                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": result_str
                            }
                        ]
                    }
                }))
            } else if tool_name == "calculate_mersenne_prime" {
                let n = payload
                    .params
                    .as_ref()
                    .and_then(|p| {
                        p.get("arguments")
                            .and_then(|a| a.get("n").and_then(|v| v.as_i64()))
                    })
                    .unwrap_or(0);

                tracing::info!("MCP tool calculate_mersenne_prime called with n: {}", n);

                let result_str = match run_a2a_benchmark(n).await {
                    Ok(res) => res,
                    Err(err) => format!("Error in Master Coordinator Benchmark: {}", err),
                };

                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": result_str
                            }
                        ]
                    }
                }))
            } else if tool_name == "check_agents_status" {
                tracing::info!("MCP tool check_agents_status called");
                let report = check_sub_agents_health().await;
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            {
                                "type": "text",
                                "text": report
                            }
                        ]
                    }
                }))
            } else {
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": format!("Tool not found: {}", tool_name)
                    }
                }))
            }
        }
        _ => Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", payload.method)
            }
        })),
    }
}

/// Receives POST messages from the MCP client in HTTP transport mode (Synchronous JSON-RPC).
async fn handle_mcp_message(
    Json(payload): Json<McpPostMessage>,
) -> impl axum::response::IntoResponse {
    match process_mcp_message_raw(payload).await {
        Some(response) => axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&response).unwrap(),
            ))
            .unwrap(),
        None => axum::response::Response::builder()
            .status(axum::http::StatusCode::NO_CONTENT)
            .body(axum::body::Body::empty())
            .unwrap(),
    }
}

/// Runs MCP server communicating over standard input/output (stdio).
async fn run_stdio_mcp() {
    use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = io::stdout();

    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        let payload: Result<McpPostMessage, _> = serde_json::from_str(&line);
        match payload {
            Ok(msg) => {
                if let Some(response) = process_mcp_message_raw(msg).await {
                    let response_str = serde_json::to_string(&response).unwrap();
                    let _ = stdout.write_all(response_str.as_bytes()).await;
                    let _ = stdout.write_all(b"\n").await;
                    let _ = stdout.flush().await;
                }
            }
            Err(e) => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                let response_str = serde_json::to_string(&response).unwrap();
                let _ = stdout.write_all(response_str.as_bytes()).await;
                let _ = stdout.write_all(b"\n").await;
                let _ = stdout.flush().await;
            }
        }
    }
}

// ============================================================================
// Main Execution
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize subscriber for structured logging, directing to stderr
    // to prevent corrupting stdio transport (stdout).
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--stdio".to_string()) {
        run_stdio_mcp().await;
        return;
    }

    let model_name = std::env::var("MODEL_NAME").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
    tracing::info!(
        "Rust Master Agent configured with MODEL_NAME: {}",
        model_name
    );

    let app = Router::new()
        // A2A Protocol Routes
        .route("/.well-known/agent-card.json", get(get_agent_card))
        .route("/.well-known/agent.json", get(get_agent_card))
        .route("/", post(handle_rpc))
        // MCP Protocol Route (HTTP transport - POST /mcp and POST /mcp/)
        .route("/mcp", post(handle_mcp_message))
        .route("/mcp/", post(handle_mcp_message));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8100".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!(
        "🚀 Rust Master A2A and MCP Server started on http://{}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
