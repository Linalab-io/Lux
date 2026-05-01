use std::{
    collections::{HashMap, VecDeque},
    path::Path,
    sync::Arc,
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path as AxumPath, Query, State,
    },
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::protocol::{EventEnvelope, PROTOCOL_VERSION};

#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub token: String,
    pub history_capacity: usize,
}

#[derive(Clone)]
pub struct GatewayState {
    config: Arc<GatewayConfig>,
    events: broadcast::Sender<EventEnvelope>,
    history: Arc<Mutex<VecDeque<EventEnvelope>>>,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    pipelines: Arc<Mutex<HashMap<String, PipelineRun>>>,
    graphs: Arc<Mutex<HashMap<String, StoredGraph>>>,
    tools: Arc<Mutex<HashMap<String, ToolSession>>>,
    tool_executions: Arc<Mutex<HashMap<String, ToolExecution>>>,
}

#[derive(Debug, Deserialize)]
struct SocketQuery {
    role: Option<String>,
    client_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
    protocol_version: u32,
    websocket_path: &'static str,
    history_capacity: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at_utc: String,
    pub updated_at_utc: String,
    pub status: SessionStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionStatus {
    Active,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSessionRequest {
    name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineRun {
    pub id: String,
    pub kind: String,
    pub session_id: Option<String>,
    pub status: PipelineStatus,
    pub created_at_utc: String,
    pub updated_at_utc: String,
    pub request: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PipelineStatus {
    Queued,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecutePipelineRequest {
    kind: Option<String>,
    session_id: Option<String>,
    #[serde(default)]
    request: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredGraph {
    pub id: String,
    pub display_name: String,
    pub schema_version: String,
    pub nodes: Vec<serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateGraphRequest {
    display_name: Option<String>,
    schema_version: Option<String>,
    nodes: Option<Vec<serde_json::Value>>,
    edges: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateGraphRequest {
    display_name: Option<String>,
    nodes: Option<Vec<serde_json::Value>>,
    edges: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteGraphRequest {
    #[serde(default)]
    request: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSession {
    pub id: String,
    pub tool_type: String,
    pub status: ToolConnectionStatus,
    pub created_at_utc: String,
    pub updated_at_utc: String,
    pub command_history: Vec<ToolCommandEntry>,
    pub last_output: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCommandEntry {
    pub id: String,
    pub command: String,
    pub timestamp: String,
    pub output_preview: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolExecution {
    pub id: String,
    pub tool_session_id: String,
    pub command: String,
    pub status: ToolExecutionStatus,
    pub created_at_utc: String,
    pub updated_at_utc: String,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteToolRequest {
    tool_type: String,
    command: String,
    session_id: Option<String>,
    skill_name: Option<String>,
    skill_params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateToolSessionRequest {
    tool_type: String,
}

impl GatewayState {
    pub fn new(config: GatewayConfig) -> Self {
        let (events, _) = broadcast::channel(config.history_capacity.max(1));
        Self {
            config: Arc::new(config),
            events,
            history: Arc::new(Mutex::new(VecDeque::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            graphs: Arc::new(Mutex::new(HashMap::new())),
            tools: Arc::new(Mutex::new(HashMap::new())),
            tool_executions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn accepts_token(&self, supplied: Option<&str>) -> bool {
        supplied
            .filter(|value| !value.is_empty())
            .is_some_and(|value| value == self.config.token)
    }

    async fn record_event(&self, event: EventEnvelope) {
        let mut history = self.history.lock().await;
        while history.len() >= self.config.history_capacity.max(1) {
            history.pop_front();
        }
        history.push_back(event);
    }

    async fn history_snapshot(&self) -> Vec<EventEnvelope> {
        self.history.lock().await.iter().cloned().collect()
    }
}

pub fn router(state: GatewayState) -> Router {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui");

    Router::new()
        .route("/health", get(health))
        .route("/schema", get(schema))
        .route("/events", get(events_socket))
        .route("/api/sessions", get(list_sessions).post(create_session))
        .route(
            "/api/sessions/:session_id",
            get(get_session).delete(delete_session),
        )
        .route("/api/tools", get(list_available_tools))
        .route(
            "/api/tools/sessions",
            get(list_tool_sessions).post(create_tool_session),
        )
        .route(
            "/api/tools/sessions/:session_id",
            get(get_tool_session).delete(delete_tool_session),
        )
        .route("/api/tools/execute", post(execute_tool_command))
        .route(
            "/api/tools/executions/:execution_id",
            get(get_tool_execution),
        )
        .route(
            "/api/pipeline",
            get(list_pipeline_runs).post(execute_pipeline),
        )
        .route("/api/pipeline/:run_id", get(get_pipeline_run))
        .route("/api/graphs", get(list_graphs).post(create_graph))
        .route(
            "/api/graphs/:graph_id",
            get(get_graph).put(update_graph).delete(delete_graph),
        )
        .route(
            "/api/graphs/:graph_id/execute",
            axum::routing::post(execute_graph),
        )
        .route("/api/node-types", get(list_node_types))
        .nest_service(
            "/ui",
            ServeDir::new(ui_dir).append_index_html_on_directories(true),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<GatewayState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        protocol_version: PROTOCOL_VERSION,
        websocket_path: "/events",
        history_capacity: state.config.history_capacity,
    })
}

async fn schema() -> Json<EventEnvelope> {
    Json(EventEnvelope::schema_example())
}

async fn list_sessions(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Session>>, Response> {
    require_token(&state, &headers)?;
    let mut sessions: Vec<_> = state.sessions.lock().await.values().cloned().collect();
    sessions.sort_by(|left, right| left.created_at_utc.cmp(&right.created_at_utc));
    Ok(Json(sessions))
}

async fn create_session(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<Session>), Response> {
    require_token(&state, &headers)?;

    let now = chrono_like_now();
    let session = Session {
        id: Uuid::new_v4().to_string(),
        name: request.name.unwrap_or_else(|| "Lux Session".to_string()),
        created_at_utc: now.clone(),
        updated_at_utc: now,
        status: SessionStatus::Active,
    };

    state
        .sessions
        .lock()
        .await
        .insert(session.id.clone(), session.clone());

    Ok((StatusCode::CREATED, Json(session)))
}

async fn get_session(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(session_id): AxumPath<String>,
) -> Result<Json<Session>, Response> {
    require_token(&state, &headers)?;
    state
        .sessions
        .lock()
        .await
        .get(&session_id)
        .cloned()
        .map(Json)
        .ok_or_else(not_found)
}

async fn delete_session(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(session_id): AxumPath<String>,
) -> Result<StatusCode, Response> {
    require_token(&state, &headers)?;
    if state.sessions.lock().await.remove(&session_id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found())
    }
}

async fn list_available_tools() -> Json<serde_json::Value> {
    Json(serde_json::json!([
        {
            "type": "claude-code",
            "displayName": "Claude Code",
            "description": "Anthropic Claude Code CLI integration for AI-assisted coding.",
            "integrationMethod": "cli",
            "capabilities": ["code-generation", "code-analysis", "skill-dispatch"],
            "status": "available"
        },
        {
            "type": "openai-codex",
            "displayName": "OpenAI Codex",
            "description": "OpenAI Codex image generation and code generation backend.",
            "integrationMethod": "cli",
            "capabilities": ["image-generation", "code-generation", "skill-dispatch"],
            "status": "available"
        },
        {
            "type": "opencode",
            "displayName": "OpenCode",
            "description": "OpenCode AI coding agent with MCP and skill support.",
            "integrationMethod": "mcp",
            "capabilities": ["code-generation", "code-analysis", "skill-dispatch", "mcp-tools"],
            "status": "available"
        }
    ]))
}

async fn list_tool_sessions(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ToolSession>>, Response> {
    require_token(&state, &headers)?;
    let mut sessions: Vec<_> = state.tools.lock().await.values().cloned().collect();
    sessions.sort_by(|left, right| left.updated_at_utc.cmp(&right.updated_at_utc));
    Ok(Json(sessions))
}

async fn create_tool_session(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<CreateToolSessionRequest>,
) -> Result<(StatusCode, Json<ToolSession>), Response> {
    require_token(&state, &headers)?;

    let now = chrono_like_now();
    let session = ToolSession {
        id: Uuid::new_v4().to_string(),
        tool_type: request.tool_type,
        status: ToolConnectionStatus::Connected,
        created_at_utc: now.clone(),
        updated_at_utc: now,
        command_history: Vec::new(),
        last_output: None,
    };

    state
        .tools
        .lock()
        .await
        .insert(session.id.clone(), session.clone());

    Ok((StatusCode::CREATED, Json(session)))
}

async fn get_tool_session(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(session_id): AxumPath<String>,
) -> Result<Json<ToolSession>, Response> {
    require_token(&state, &headers)?;
    state
        .tools
        .lock()
        .await
        .get(&session_id)
        .cloned()
        .map(Json)
        .ok_or_else(not_found)
}

async fn delete_tool_session(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(session_id): AxumPath<String>,
) -> Result<StatusCode, Response> {
    require_token(&state, &headers)?;
    if state.tools.lock().await.remove(&session_id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found())
    }
}

async fn execute_tool_command(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<ExecuteToolRequest>,
) -> Result<(StatusCode, Json<ToolExecution>), Response> {
    require_token(&state, &headers)?;

    let now = chrono_like_now();
    let session_id = ensure_tool_session(&state, &request, &now).await;
    let execution = ToolExecution {
        id: Uuid::new_v4().to_string(),
        tool_session_id: session_id.clone(),
        command: request.command.clone(),
        status: ToolExecutionStatus::Running,
        created_at_utc: now.clone(),
        updated_at_utc: now.clone(),
        output: None,
        error: None,
    };

    state
        .tool_executions
        .lock()
        .await
        .insert(execution.id.clone(), execution.clone());

    record_tool_command(&state, &session_id, &request.command, &now).await;

    let payload = if let Some(skill_name) = request.skill_name {
        serde_json::json!({
            "kind": "skill-dispatch",
            "toolType": request.tool_type,
            "skillName": skill_name,
            "skillParams": request.skill_params.unwrap_or_else(|| serde_json::json!({})),
            "executionId": execution.id.clone(),
        })
    } else {
        serde_json::json!({
            "kind": "tool-execute",
            "toolType": request.tool_type,
            "command": request.command,
            "executionId": execution.id.clone(),
        })
    };

    let event = EventEnvelope {
        schema_version: PROTOCOL_VERSION,
        event_id: Uuid::new_v4().to_string(),
        category: crate::protocol::EventCategory::Tool,
        source: "lux-gateway".to_string(),
        session_id,
        captured_at_utc: now,
        payload,
    };

    publish_event(&state, event).await;
    Ok((StatusCode::ACCEPTED, Json(execution)))
}

async fn get_tool_execution(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(execution_id): AxumPath<String>,
) -> Result<Json<ToolExecution>, Response> {
    require_token(&state, &headers)?;
    state
        .tool_executions
        .lock()
        .await
        .get(&execution_id)
        .cloned()
        .map(Json)
        .ok_or_else(not_found)
}

async fn ensure_tool_session(
    state: &GatewayState,
    request: &ExecuteToolRequest,
    now: &str,
) -> String {
    let mut sessions = state.tools.lock().await;
    if let Some(session_id) = &request.session_id {
        if sessions.contains_key(session_id) {
            return session_id.clone();
        }
    }

    let session = ToolSession {
        id: Uuid::new_v4().to_string(),
        tool_type: request.tool_type.clone(),
        status: ToolConnectionStatus::Connected,
        created_at_utc: now.to_string(),
        updated_at_utc: now.to_string(),
        command_history: Vec::new(),
        last_output: None,
    };
    let session_id = session.id.clone();
    sessions.insert(session_id.clone(), session);
    session_id
}

async fn record_tool_command(
    state: &GatewayState,
    session_id: &str,
    command: &str,
    now: &str,
) {
    let mut sessions = state.tools.lock().await;
    if let Some(session) = sessions.get_mut(session_id) {
        session.updated_at_utc = now.to_string();
        session.command_history.push(ToolCommandEntry {
            id: Uuid::new_v4().to_string(),
            command: command.to_string(),
            timestamp: now.to_string(),
            output_preview: None,
        });
    }
}

async fn list_pipeline_runs(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Json<Vec<PipelineRun>>, Response> {
    require_token(&state, &headers)?;
    let mut runs: Vec<_> = state.pipelines.lock().await.values().cloned().collect();
    runs.sort_by(|left, right| left.created_at_utc.cmp(&right.created_at_utc));
    Ok(Json(runs))
}

async fn execute_pipeline(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<ExecutePipelineRequest>,
) -> Result<(StatusCode, Json<PipelineRun>), Response> {
    require_token(&state, &headers)?;

    let now = chrono_like_now();
    let run = PipelineRun {
        id: Uuid::new_v4().to_string(),
        kind: request.kind.unwrap_or_else(|| "codex-image".to_string()),
        session_id: request.session_id,
        status: PipelineStatus::Queued,
        created_at_utc: now.clone(),
        updated_at_utc: now,
        request: request.request,
    };

    state
        .pipelines
        .lock()
        .await
        .insert(run.id.clone(), run.clone());

    Ok((StatusCode::ACCEPTED, Json(run)))
}

async fn get_pipeline_run(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(run_id): AxumPath<String>,
) -> Result<Json<PipelineRun>, Response> {
    require_token(&state, &headers)?;
    state
        .pipelines
        .lock()
        .await
        .get(&run_id)
        .cloned()
        .map(Json)
        .ok_or_else(not_found)
}

async fn list_graphs(
    State(state): State<GatewayState>,
    headers: HeaderMap,
) -> Result<Json<Vec<StoredGraph>>, Response> {
    require_token(&state, &headers)?;
    let mut graphs: Vec<_> = state.graphs.lock().await.values().cloned().collect();
    graphs.sort_by(|left, right| left.created_at_utc.cmp(&right.created_at_utc));
    Ok(Json(graphs))
}

async fn create_graph(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    Json(request): Json<CreateGraphRequest>,
) -> Result<(StatusCode, Json<StoredGraph>), Response> {
    require_token(&state, &headers)?;

    let now = chrono_like_now();
    let graph = StoredGraph {
        id: Uuid::new_v4().to_string(),
        display_name: request
            .display_name
            .unwrap_or_else(|| "Lux Pipeline Graph".to_string()),
        schema_version: request.schema_version.unwrap_or_else(|| "1.0".to_string()),
        nodes: request.nodes.unwrap_or_default(),
        edges: request.edges.unwrap_or_default(),
        created_at_utc: now.clone(),
        updated_at_utc: now,
    };

    state
        .graphs
        .lock()
        .await
        .insert(graph.id.clone(), graph.clone());

    Ok((StatusCode::CREATED, Json(graph)))
}

async fn get_graph(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(graph_id): AxumPath<String>,
) -> Result<Json<StoredGraph>, Response> {
    require_token(&state, &headers)?;
    state
        .graphs
        .lock()
        .await
        .get(&graph_id)
        .cloned()
        .map(Json)
        .ok_or_else(not_found)
}

async fn update_graph(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(graph_id): AxumPath<String>,
    Json(request): Json<UpdateGraphRequest>,
) -> Result<Json<StoredGraph>, Response> {
    require_token(&state, &headers)?;

    let mut graphs = state.graphs.lock().await;
    let graph = graphs.get_mut(&graph_id).ok_or_else(not_found)?;
    if let Some(display_name) = request.display_name {
        graph.display_name = display_name;
    }
    if let Some(nodes) = request.nodes {
        graph.nodes = nodes;
    }
    if let Some(edges) = request.edges {
        graph.edges = edges;
    }
    graph.updated_at_utc = chrono_like_now();

    Ok(Json(graph.clone()))
}

async fn delete_graph(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(graph_id): AxumPath<String>,
) -> Result<StatusCode, Response> {
    require_token(&state, &headers)?;
    if state.graphs.lock().await.remove(&graph_id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found())
    }
}

async fn execute_graph(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    AxumPath(graph_id): AxumPath<String>,
    Json(request): Json<ExecuteGraphRequest>,
) -> Result<(StatusCode, Json<EventEnvelope>), Response> {
    require_token(&state, &headers)?;
    let graph = state
        .graphs
        .lock()
        .await
        .get(&graph_id)
        .cloned()
        .ok_or_else(not_found)?;

    let event = EventEnvelope {
        schema_version: PROTOCOL_VERSION,
        event_id: Uuid::new_v4().to_string(),
        category: crate::protocol::EventCategory::Tool,
        source: "lux".to_string(),
        session_id: graph.id.clone(),
        captured_at_utc: chrono_like_now(),
        payload: serde_json::json!({
            "kind": "execute-graph",
            "graph": graph,
            "request": request.request,
        }),
    };

    publish_event(&state, event.clone()).await;
    Ok((StatusCode::ACCEPTED, Json(event)))
}

async fn list_node_types() -> Json<serde_json::Value> {
    Json(serde_json::json!([
        {
            "type": "unity-context",
            "displayName": "Unity Context",
            "description": "Exports scene, selection, and editor state from the active project.",
            "category": "context",
            "inputPorts": [],
            "outputPorts": [
                { "name": "context", "direction": "output", "dataType": "unity-context" }
            ],
            "parameters": []
        },
        {
            "type": "output-directory",
            "displayName": "Output Directory",
            "description": "Package-local destination for generated sprites and masks.",
            "category": "context",
            "inputPorts": [],
            "outputPorts": [
                { "name": "outputDirectory", "direction": "output", "dataType": "output-directory" }
            ],
            "parameters": [
                { "name": "path", "type": "string", "description": "Project-relative output path." },
                { "name": "allowLocalUserOverride", "type": "boolean", "description": "Allow absolute paths." }
            ]
        },
        {
            "type": "prompt-template",
            "displayName": "Prompt",
            "description": "Combines Unity context with reusable Codex Image prompts.",
            "category": "pipeline",
            "inputPorts": [
                { "name": "context", "direction": "input", "dataType": "unity-context" },
                { "name": "outputDirectory", "direction": "input", "dataType": "output-directory" }
            ],
            "outputPorts": [
                { "name": "prompt", "direction": "output", "dataType": "prompt" },
                { "name": "manifest", "direction": "output", "dataType": "generated-asset-manifest" }
            ],
            "parameters": [
                { "name": "template", "type": "string", "description": "Inline prompt template." },
                { "name": "templatePath", "type": "string", "description": "Path to prompt template file." },
                { "name": "backendName", "type": "string", "description": "Backend name, default 'Codex'." }
            ]
        },
        {
            "type": "codex-generation",
            "displayName": "Generation",
            "description": "Queues an AI image generation job through Lux tooling.",
            "category": "pipeline",
            "inputPorts": [
                { "name": "prompt", "direction": "input", "dataType": "prompt" },
                { "name": "manifest", "direction": "input", "dataType": "generated-asset-manifest" }
            ],
            "outputPorts": [
                { "name": "generatedAssets", "direction": "output", "dataType": "generated-asset-manifest" }
            ],
            "parameters": []
        },
        {
            "type": "segmentation",
            "displayName": "Segmentation",
            "description": "Separates subject, mask, and background layers.",
            "category": "post-process",
            "inputPorts": [
                { "name": "generatedAssets", "direction": "input", "dataType": "generated-asset-manifest" }
            ],
            "outputPorts": [
                { "name": "segmentationResponse", "direction": "output", "dataType": "segmentation-response" }
            ],
            "parameters": []
        },
        {
            "type": "mask-post-processing",
            "displayName": "Export",
            "description": "Cleans masks and prepares Unity-ready assets.",
            "category": "post-process",
            "inputPorts": [
                { "name": "segmentationResponse", "direction": "input", "dataType": "segmentation-response" }
            ],
            "outputPorts": [],
            "parameters": []
        }
    ]))
}

async fn events_socket(
    State(state): State<GatewayState>,
    Query(query): Query<SocketQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let token = headers
        .get("x-lux-token")
        .and_then(|value| value.to_str().ok());

    if !state.accepts_token(token) {
        return (
            StatusCode::UNAUTHORIZED,
            "invalid or missing Lux gateway token",
        )
            .into_response();
    }

    if !accepts_origin(&headers) {
        return (
            StatusCode::FORBIDDEN,
            "forbidden Lux gateway WebSocket origin",
        )
            .into_response();
    }

    let role = query.role.unwrap_or_else(|| "subscriber".to_string());
    let client_id = query
        .client_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    ws.on_upgrade(move |socket| handle_socket(state, socket, role, client_id))
}

async fn handle_socket(state: GatewayState, socket: WebSocket, role: String, client_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut events = state.events.subscribe();

    for event in state.history_snapshot().await {
        if send_event(&mut sender, &event).await.is_err() {
            return;
        }
    }

    let connected = EventEnvelope {
        schema_version: PROTOCOL_VERSION,
        event_id: Uuid::new_v4().to_string(),
        category: crate::protocol::EventCategory::Tool,
        source: "lux".to_string(),
        session_id: client_id.clone(),
        captured_at_utc: chrono_like_now(),
        payload: serde_json::json!({
            "kind": "client-connected",
            "role": role,
            "clientId": client_id,
        }),
    };
    publish_event(&state, connected).await;

    loop {
        tokio::select! {
            received = events.recv() => {
                match received {
                    Ok(event) => {
                        if send_event(&mut sender, &event).await.is_err() {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(%skipped, "Lux gateway subscriber lagged behind");
                    }
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
            message = receiver.next() => {
                match message {
                    Some(Ok(Message::Text(text))) => {
                        if text.len() > 64 * 1024 {
                            tracing::warn!("Lux gateway ignored oversized event envelope");
                            continue;
                        }

                        match serde_json::from_str::<EventEnvelope>(&text) {
                            Ok(event) => publish_event(&state, event.normalize()).await,
                            Err(error) => tracing::warn!(%error, "Lux gateway ignored malformed event envelope"),
                        }
                    },
                    Some(Ok(Message::Close(_))) | None => return,
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        tracing::warn!(%error, "Lux gateway WebSocket error");
                        return;
                    }
                }
            }
        }
    }
}

fn accepts_origin(headers: &HeaderMap) -> bool {
    let Some(origin) = headers.get("origin").and_then(|value| value.to_str().ok()) else {
        return true;
    };

    if origin == "null" {
        return true;
    }

    let Ok(uri) = origin.parse::<Uri>() else {
        return false;
    };

    matches!(uri.scheme_str(), Some("http") | Some("https"))
        && matches!(
            uri.host(),
            Some("localhost") | Some("127.0.0.1") | Some("::1")
        )
}

struct AuthError;

impl From<AuthError> for Response {
    fn from(_: AuthError) -> Self {
        (
            StatusCode::UNAUTHORIZED,
            "invalid or missing Lux gateway token",
        )
            .into_response()
    }
}

fn require_token(state: &GatewayState, headers: &HeaderMap) -> Result<(), AuthError> {
    let token = headers
        .get("x-lux-token")
        .and_then(|value| value.to_str().ok());

    if state.accepts_token(token) {
        Ok(())
    } else {
        Err(AuthError)
    }
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Lux gateway resource not found").into_response()
}

async fn publish_event(state: &GatewayState, event: EventEnvelope) {
    state.record_event(event.clone()).await;
    let _ = state.events.send(event);
}

async fn send_event(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    event: &EventEnvelope,
) -> Result<(), axum::Error> {
    sender
        .send(Message::Text(
            serde_json::to_string(event).unwrap_or_default(),
        ))
        .await
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use http::{header, Method, Request};
    use tower::ServiceExt;

    fn test_app() -> Router {
        router(GatewayState::new(GatewayConfig {
            token: "secret".to_string(),
            history_capacity: 8,
        }))
    }

    async fn json_request(
        app: Router,
        method: Method,
        uri: &str,
        body: serde_json::Value,
    ) -> Response {
        app.oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("x-lux-token", "secret")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn authenticated_get(app: Router, uri: &str) -> Response {
        app.oneshot(
            Request::builder()
                .uri(uri)
                .header("x-lux-token", "secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn response_json(response: Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[test]
    fn token_validation_requires_exact_match() {
        let state = GatewayState::new(GatewayConfig {
            token: "secret".to_string(),
            history_capacity: 8,
        });

        assert!(state.accepts_token(Some("secret")));
        assert!(!state.accepts_token(Some("SECRET")));
        assert!(!state.accepts_token(Some("")));
        assert!(!state.accepts_token(None));
    }

    #[test]
    fn origin_validation_allows_localhost_and_rejects_remote_origins() {
        let mut headers = HeaderMap::new();
        assert!(accepts_origin(&headers));

        headers.insert("origin", "http://127.0.0.1:3000".parse().unwrap());
        assert!(accepts_origin(&headers));

        headers.insert("origin", "http://localhost:3000".parse().unwrap());
        assert!(accepts_origin(&headers));

        headers.insert("origin", "https://evil.example".parse().unwrap());
        assert!(!accepts_origin(&headers));

        headers.insert("origin", "http://localhost.evil.example".parse().unwrap());
        assert!(!accepts_origin(&headers));

        headers.insert("origin", "http://127.0.0.1.evil.example".parse().unwrap());
        assert!(!accepts_origin(&headers));
    }

    #[tokio::test]
    async fn history_respects_capacity() {
        let state = GatewayState::new(GatewayConfig {
            token: "secret".to_string(),
            history_capacity: 2,
        });

        for index in 0..3 {
            state
                .record_event(EventEnvelope {
                    schema_version: PROTOCOL_VERSION,
                    event_id: format!("event-{index}"),
                    category: crate::protocol::EventCategory::Log,
                    source: "test".to_string(),
                    session_id: "test-session".to_string(),
                    captured_at_utc: "test-time".to_string(),
                    payload: serde_json::json!({ "index": index }),
                })
                .await;
        }

        let history = state.history_snapshot().await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].event_id, "event-1");
        assert_eq!(history[1].event_id, "event-2");
    }

    #[tokio::test]
    async fn ui_serves_index_html() {
        let response = test_app()
            .oneshot(Request::builder().uri("/ui/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let html = std::str::from_utf8(&body).unwrap();
        assert!(html.contains("Lux Gateway"));
    }

    #[tokio::test]
    async fn session_crud_requires_token_and_persists_sessions() {
        let app = test_app();

        let unauthorized = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/sessions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let created = json_request(
            app.clone(),
            Method::POST,
            "/api/sessions",
            serde_json::json!({ "name": "Codex asset pass" }),
        )
        .await;
        assert_eq!(created.status(), StatusCode::CREATED);
        let created_json = response_json(created).await;
        let session_id = created_json["id"].as_str().unwrap();
        assert_eq!(created_json["name"], "Codex asset pass");
        assert_eq!(created_json["status"], "active");

        let fetched = authenticated_get(app.clone(), &format!("/api/sessions/{session_id}")).await;
        assert_eq!(fetched.status(), StatusCode::OK);
        assert_eq!(response_json(fetched).await["id"], session_id);

        let listed = authenticated_get(app.clone(), "/api/sessions").await;
        assert_eq!(listed.status(), StatusCode::OK);
        assert_eq!(response_json(listed).await.as_array().unwrap().len(), 1);

        let deleted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/sessions/{session_id}"))
                    .header("x-lux-token", "secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

        let missing = authenticated_get(app, &format!("/api/sessions/{session_id}")).await;
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn pipeline_execute_list_and_status() {
        let app = test_app();

        let created = json_request(
            app.clone(),
            Method::POST,
            "/api/pipeline",
            serde_json::json!({
                "kind": "codex-image",
                "sessionId": "session-1",
                "request": { "prompt": "neon sprite" }
            }),
        )
        .await;
        assert_eq!(created.status(), StatusCode::ACCEPTED);
        let created_json = response_json(created).await;
        let run_id = created_json["id"].as_str().unwrap();
        assert_eq!(created_json["kind"], "codex-image");
        assert_eq!(created_json["status"], "queued");
        assert_eq!(created_json["request"]["prompt"], "neon sprite");

        let listed = authenticated_get(app.clone(), "/api/pipeline").await;
        assert_eq!(listed.status(), StatusCode::OK);
        assert_eq!(response_json(listed).await.as_array().unwrap().len(), 1);

        let fetched = authenticated_get(app, &format!("/api/pipeline/{run_id}")).await;
        assert_eq!(fetched.status(), StatusCode::OK);
        assert_eq!(response_json(fetched).await["id"], run_id);
    }

    #[tokio::test]
    async fn graph_crud_full_lifecycle() {
        let app = test_app();

        let created = json_request(
            app.clone(),
            Method::POST,
            "/api/graphs",
            serde_json::json!({
                "displayName": "Codex Image Graph",
                "schemaVersion": "1.0",
                "nodes": [{ "id": "node-1", "type": "unity-context" }],
                "edges": []
            }),
        )
        .await;
        assert_eq!(created.status(), StatusCode::CREATED);
        let created_json = response_json(created).await;
        let graph_id = created_json["id"].as_str().unwrap();
        assert_eq!(created_json["displayName"], "Codex Image Graph");
        assert_eq!(created_json["schemaVersion"], "1.0");
        assert_eq!(created_json["nodes"].as_array().unwrap().len(), 1);

        let fetched = authenticated_get(app.clone(), &format!("/api/graphs/{graph_id}")).await;
        assert_eq!(fetched.status(), StatusCode::OK);
        assert_eq!(response_json(fetched).await["id"], graph_id);

        let listed = authenticated_get(app.clone(), "/api/graphs").await;
        assert_eq!(listed.status(), StatusCode::OK);
        assert_eq!(response_json(listed).await.as_array().unwrap().len(), 1);

        let updated = json_request(
            app.clone(),
            Method::PUT,
            &format!("/api/graphs/{graph_id}"),
            serde_json::json!({
                "displayName": "Updated Graph",
                "nodes": [{ "id": "node-2", "type": "prompt-template" }],
                "edges": [{ "from": "node-1", "to": "node-2" }]
            }),
        )
        .await;
        assert_eq!(updated.status(), StatusCode::OK);
        let updated_json = response_json(updated).await;
        assert_eq!(updated_json["displayName"], "Updated Graph");
        assert_eq!(updated_json["nodes"][0]["id"], "node-2");
        assert_eq!(updated_json["edges"].as_array().unwrap().len(), 1);

        let deleted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/graphs/{graph_id}"))
                    .header("x-lux-token", "secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

        let missing = authenticated_get(app, &format!("/api/graphs/{graph_id}")).await;
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn graph_execute_broadcasts_tool_event() {
        let state = GatewayState::new(GatewayConfig {
            token: "secret".to_string(),
            history_capacity: 8,
        });
        let app = router(state.clone());

        let created = json_request(
            app.clone(),
            Method::POST,
            "/api/graphs",
            serde_json::json!({
                "displayName": "Executable Graph",
                "nodes": [{ "id": "node-1", "type": "codex-generation" }],
                "edges": []
            }),
        )
        .await;
        assert_eq!(created.status(), StatusCode::CREATED);
        let graph_id = response_json(created).await["id"]
            .as_str()
            .unwrap()
            .to_string();

        let mut events = state.events.subscribe();
        let executed = json_request(
            app,
            Method::POST,
            &format!("/api/graphs/{graph_id}/execute"),
            serde_json::json!({ "request": { "trigger": "test" } }),
        )
        .await;
        assert_eq!(executed.status(), StatusCode::ACCEPTED);
        let executed_json = response_json(executed).await;
        assert_eq!(executed_json["category"], "tool");
        assert_eq!(executed_json["payload"]["kind"], "execute-graph");
        assert_eq!(executed_json["payload"]["graph"]["id"], graph_id);

        let broadcast = events.recv().await.unwrap();
        assert_eq!(broadcast.category, crate::protocol::EventCategory::Tool);
        assert_eq!(broadcast.payload["kind"], "execute-graph");
        assert_eq!(broadcast.payload["graph"]["id"], graph_id);
    }

    #[tokio::test]
    async fn tool_available_tools_registry_no_auth_required() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/tools")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let tools = response_json(response).await;
        let tools = tools.as_array().unwrap();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0]["type"], "claude-code");
        assert_eq!(tools[0]["displayName"], "Claude Code");
        assert_eq!(tools[1]["type"], "openai-codex");
        assert_eq!(tools[2]["type"], "opencode");
        assert_eq!(tools[2]["integrationMethod"], "mcp");
        assert!(tools[2]["capabilities"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("mcp-tools")));
    }

    #[tokio::test]
    async fn tool_session_crud_lifecycle() {
        let app = test_app();

        let created = json_request(
            app.clone(),
            Method::POST,
            "/api/tools/sessions",
            serde_json::json!({ "toolType": "claude-code" }),
        )
        .await;
        assert_eq!(created.status(), StatusCode::CREATED);
        let created_json = response_json(created).await;
        let session_id = created_json["id"].as_str().unwrap();
        assert_eq!(created_json["toolType"], "claude-code");
        assert_eq!(created_json["status"], "connected");
        assert_eq!(created_json["commandHistory"].as_array().unwrap().len(), 0);

        let fetched = authenticated_get(app.clone(), &format!("/api/tools/sessions/{session_id}")).await;
        assert_eq!(fetched.status(), StatusCode::OK);
        assert_eq!(response_json(fetched).await["id"], session_id);

        let listed = authenticated_get(app.clone(), "/api/tools/sessions").await;
        assert_eq!(listed.status(), StatusCode::OK);
        assert_eq!(response_json(listed).await.as_array().unwrap().len(), 1);

        let deleted = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/tools/sessions/{session_id}"))
                    .header("x-lux-token", "secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(deleted.status(), StatusCode::NO_CONTENT);

        let missing = authenticated_get(app, &format!("/api/tools/sessions/{session_id}")).await;
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn tool_execute_broadcasts_tool_execute_event() {
        let state = GatewayState::new(GatewayConfig {
            token: "secret".to_string(),
            history_capacity: 8,
        });
        let app = router(state.clone());

        let mut events = state.events.subscribe();
        let executed = json_request(
            app.clone(),
            Method::POST,
            "/api/tools/execute",
            serde_json::json!({
                "toolType": "claude-code",
                "command": "fix the compile error in Player.cs"
            }),
        )
        .await;
        assert_eq!(executed.status(), StatusCode::ACCEPTED);
        let executed_json = response_json(executed).await;
        let execution_id = executed_json["id"].as_str().unwrap();
        let session_id = executed_json["toolSessionId"].as_str().unwrap();
        assert_eq!(executed_json["status"], "running");
        assert_eq!(executed_json["command"], "fix the compile error in Player.cs");

        let broadcast = events.recv().await.unwrap();
        assert_eq!(broadcast.category, crate::protocol::EventCategory::Tool);
        assert_eq!(broadcast.source, "lux-gateway");
        assert_eq!(broadcast.session_id, session_id);
        assert_eq!(broadcast.payload["kind"], "tool-execute");
        assert_eq!(broadcast.payload["toolType"], "claude-code");
        assert_eq!(broadcast.payload["command"], "fix the compile error in Player.cs");
        assert_eq!(broadcast.payload["executionId"], execution_id);

        let fetched = authenticated_get(app.clone(), &format!("/api/tools/executions/{execution_id}")).await;
        assert_eq!(fetched.status(), StatusCode::OK);
        assert_eq!(response_json(fetched).await["id"], execution_id);

        let session = authenticated_get(app, &format!("/api/tools/sessions/{session_id}")).await;
        assert_eq!(session.status(), StatusCode::OK);
        let session_json = response_json(session).await;
        assert_eq!(session_json["commandHistory"].as_array().unwrap().len(), 1);
        assert_eq!(
            session_json["commandHistory"][0]["command"],
            "fix the compile error in Player.cs"
        );
    }

    #[tokio::test]
    async fn tool_skill_dispatch_broadcasts_skill_event() {
        let state = GatewayState::new(GatewayConfig {
            token: "secret".to_string(),
            history_capacity: 8,
        });
        let app = router(state.clone());

        let created = json_request(
            app.clone(),
            Method::POST,
            "/api/tools/sessions",
            serde_json::json!({ "toolType": "opencode" }),
        )
        .await;
        assert_eq!(created.status(), StatusCode::CREATED);
        let session_id = response_json(created).await["id"].as_str().unwrap().to_string();

        let mut events = state.events.subscribe();
        let executed = json_request(
            app,
            Method::POST,
            "/api/tools/execute",
            serde_json::json!({
                "toolType": "opencode",
                "command": "compile",
                "sessionId": session_id,
                "skillName": "compile",
                "skillParams": { "target": "editor" }
            }),
        )
        .await;
        assert_eq!(executed.status(), StatusCode::ACCEPTED);
        let execution_id = response_json(executed).await["id"].as_str().unwrap().to_string();

        let broadcast = events.recv().await.unwrap();
        assert_eq!(broadcast.category, crate::protocol::EventCategory::Tool);
        assert_eq!(broadcast.source, "lux-gateway");
        assert_eq!(broadcast.session_id, session_id);
        assert_eq!(broadcast.payload["kind"], "skill-dispatch");
        assert_eq!(broadcast.payload["toolType"], "opencode");
        assert_eq!(broadcast.payload["skillName"], "compile");
        assert_eq!(broadcast.payload["skillParams"]["target"], "editor");
        assert_eq!(broadcast.payload["executionId"], execution_id);
    }

    #[tokio::test]
    async fn tool_endpoints_require_token() {
        let app = test_app();

        for (method, uri, body) in [
            (Method::GET, "/api/tools/sessions", serde_json::Value::Null),
            (
                Method::POST,
                "/api/tools/sessions",
                serde_json::json!({ "toolType": "claude-code" }),
            ),
            (
                Method::GET,
                "/api/tools/sessions/missing",
                serde_json::Value::Null,
            ),
            (
                Method::DELETE,
                "/api/tools/sessions/missing",
                serde_json::Value::Null,
            ),
            (
                Method::POST,
                "/api/tools/execute",
                serde_json::json!({ "toolType": "claude-code", "command": "test" }),
            ),
            (
                Method::GET,
                "/api/tools/executions/missing",
                serde_json::Value::Null,
            ),
        ] {
            let mut builder = Request::builder().method(method).uri(uri);
            let request = if body.is_null() {
                builder.body(Body::empty()).unwrap()
            } else {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                builder.body(Body::from(body.to_string())).unwrap()
            };
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn node_types_returns_static_registry() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .uri("/api/node-types")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let node_types = response_json(response).await;
        let node_types = node_types.as_array().unwrap();
        assert_eq!(node_types.len(), 6);
        assert_eq!(node_types[0]["type"], "unity-context");
        assert_eq!(node_types[1]["type"], "output-directory");
        assert_eq!(node_types[2]["type"], "prompt-template");
        assert_eq!(node_types[3]["type"], "codex-generation");
        assert_eq!(node_types[4]["type"], "segmentation");
        assert_eq!(node_types[5]["type"], "mask-post-processing");
        assert_eq!(node_types[2]["inputPorts"].as_array().unwrap().len(), 2);
        assert_eq!(node_types[5]["outputPorts"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn graph_endpoints_require_token() {
        let app = test_app();

        for (method, uri, body) in [
            (Method::GET, "/api/graphs", serde_json::Value::Null),
            (Method::POST, "/api/graphs", serde_json::json!({})),
            (Method::GET, "/api/graphs/missing", serde_json::Value::Null),
            (Method::PUT, "/api/graphs/missing", serde_json::json!({})),
            (
                Method::DELETE,
                "/api/graphs/missing",
                serde_json::Value::Null,
            ),
            (
                Method::POST,
                "/api/graphs/missing/execute",
                serde_json::json!({ "request": {} }),
            ),
        ] {
            let mut builder = Request::builder().method(method).uri(uri);
            let request = if body.is_null() {
                builder.body(Body::empty()).unwrap()
            } else {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                builder.body(Body::from(body.to_string())).unwrap()
            };
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }
}
