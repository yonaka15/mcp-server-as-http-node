use axum::{
    Json as AxumJson, // axum::Json と serde_json::Json を区別するため
    Router,
    extract::State,
    http::StatusCode,
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

// --- JSON設定ファイルの構造体 ---
#[derive(Deserialize, Debug, Clone)]
struct McpProcessConfig {
    command: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
}

type McpServersConfig = HashMap<String, McpProcessConfig>;

// --- MCPプロセスとの通信用構造体 ---
struct McpServerProcess {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    // stderr_join_handle: Option<tokio::task::JoinHandle<()>>, // 必要であればstderr監視タスクのハンドルを保持
}

impl McpServerProcess {
    async fn query(&mut self, request: &McpRequest) -> Result<McpResponse, String> {
        let request_json = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        self.stdin
            .write_all((request_json + "\n").as_bytes())
            .await
            .map_err(|e| format!("Failed to write to MCP stdin: {}", e))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush MCP stdin: {}", e))?;

        let mut response_line = String::new();
        match self.stdout.read_line(&mut response_line).await {
            Ok(0) => Err("MCP server closed the connection (EOF).".to_string()), // EOF
            Ok(_) => {
                if response_line.trim().is_empty() {
                    return Err("MCP server returned an empty line.".to_string());
                }
                serde_json::from_str::<McpResponse>(&response_line).map_err(|e| {
                    format!(
                        "Failed to deserialize MCP response: {} (raw: '{}')",
                        e,
                        response_line.trim()
                    )
                })
            }
            Err(e) => Err(format!("Failed to read from MCP stdout: {}", e)),
        }
    }
}

// --- リクエスト・レスポンスデータ構造 ---
#[derive(Serialize, Deserialize, Debug)]
struct McpRequest {
    mcp: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct McpResponse {
    result: String,
}

// --- MCPサーバープロセス起動関数 ---
async fn start_mcp_server_from_config(
    config_file_path: &str,
    server_key: &str,
) -> Result<McpServerProcess, Box<dyn std::error::Error + Send + Sync>> {
    // 1. JSON設定ファイルを読み込む
    let config_content = tokio::fs::read_to_string(config_file_path)
        .await
        .map_err(|e| {
            format!(
                "Failed to read MCP config file '{}': {}",
                config_file_path, e
            )
        })?;

    let all_configs: McpServersConfig = serde_json::from_str(&config_content).map_err(|e| {
        format!(
            "Failed to parse MCP config file '{}': {}",
            config_file_path, e
        )
    })?;

    // 2. 指定されたキーのコンフィグを取得
    let server_config = all_configs.get(server_key).ok_or_else(|| {
        format!(
            "MCP server configuration not found for key '{}' in file '{}'",
            server_key, config_file_path
        )
    })?;

    println!(
        "Starting MCP server (key: '{}') with command: '{}', args: {:?}, env: {:?}",
        server_key, &server_config.command, &server_config.args, &server_config.env
    );

    // 3. Commandを構築
    let mut command_builder = Command::new(&server_config.command);
    command_builder.args(&server_config.args);
    command_builder.envs(&server_config.env);

    command_builder
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped()); // 標準エラーもパイプする

    let mut child = command_builder.spawn().map_err(|e| {
        format!(
            "Failed to spawn MCP process for key '{}' (command: '{}'): {}",
            server_key, server_config.command, e
        )
    })?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| format!("Failed to open stdin for MCP process '{}'", server_key))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("Failed to open stdout for MCP process '{}'", server_key))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| format!("Failed to open stderr for MCP process '{}'", server_key))?;

    // MCPサーバーの標準エラー出力を非同期で読み取り、ログに出力するタスク
    let server_key_clone_for_stderr = server_key.to_string(); // stderrタスク用にキーを複製
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // eprintln!("[MCP Server stderr - {}]: EOF, task finishing.", server_key_clone_for_stderr);
                    break;
                }
                Ok(_) => {
                    eprint!(
                        "[MCP Server stderr - {}]: {}",
                        server_key_clone_for_stderr, line
                    );
                    line.clear();
                }
                Err(e) => {
                    eprintln!(
                        "[MCP Server stderr read error - {}]: {}",
                        server_key_clone_for_stderr, e
                    );
                    break;
                }
            }
        }
    });

    Ok(McpServerProcess {
        stdin,
        stdout: BufReader::new(stdout),
        // stderr_join_handle: Some(stderr_task_handle),
    })
}

// --- Axum リクエストハンドラ ---
async fn handle_mcp_request_shared(
    State(mcp_process_mutex): State<Arc<Mutex<McpServerProcess>>>,
    AxumJson(payload): AxumJson<McpRequest>, // axum::Json を使用
) -> Result<AxumJson<McpResponse>, StatusCode> {
    // Mutexロックを取得して、McpServerProcessへの可変参照を得る
    // このロックはスコープを抜けるまで保持される
    let mut mcp_process_guard = mcp_process_mutex.lock().await;

    match mcp_process_guard.query(&payload).await {
        Ok(response) => Ok(AxumJson(response)),
        Err(e) => {
            eprintln!("Error querying MCP server: {}", e);
            // ここでMCPプロセスが死んでいる可能性も考慮し、
            // より高度なエラーリカバリ（プロセス再起動など）を検討することもできる
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// --- main関数 ---
#[tokio::main]
async fn main() {
    // 環境変数から設定ファイルパスと使用するMCPサーバーのキーを取得
    let config_file =
        env::var("MCP_CONFIG_FILE").unwrap_or_else(|_| "mcp_servers.config.json".to_string());
    let mcp_server_key_to_use =
        env::var("MCP_SERVER_KEY").unwrap_or_else(|_| "brave-search".to_string()); // デフォルトのキー

    println!(
        "Attempting to start MCP server using config: '{}' and key: '{}'",
        config_file, mcp_server_key_to_use
    );

    // MCPサーバープロセスを起動
    let mcp_server_process_mutex = match start_mcp_server_from_config(
        &config_file,
        &mcp_server_key_to_use,
    )
    .await
    {
        Ok(process) => Arc::new(Mutex::new(process)),
        Err(e) => {
            eprintln!(
                "Fatal: Failed to start MCP server process from config: {}",
                e
            );
            eprintln!(
                "Please ensure the MCP server command and script paths are correct, and the script is executable."
            );
            eprintln!(
                "For example, if using 'python3 ./test_mcp_server.py', ensure './test_mcp_server.py' exists and is executable by the user running this Axum server."
            );
            return;
        }
    };

    // Axumルーターの設定
    let app = Router::new()
        .route("/api/mcp", post(handle_mcp_request_shared))
        .with_state(mcp_server_process_mutex); // MCPプロセスの通信チャネルをステートとして渡す

    let listener_addr = "127.0.0.1:3000";
    match tokio::net::TcpListener::bind(listener_addr).await {
        Ok(listener) => {
            println!(
                "Axum server listening on http://{}",
                listener.local_addr().unwrap()
            );
            if let Err(e) = axum::serve(listener, app.into_make_service()).await {
                eprintln!("Server error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to bind to address {}: {}", listener_addr, e);
        }
    }
}
