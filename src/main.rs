use axum::{
    Json as AxumJson, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc, time::Instant};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    sync::Mutex,
    time::{Duration, timeout},
};

// --- 認証設定構造体 ---
#[derive(Clone, Debug)]
struct AuthConfig {
    api_key: Option<String>,
    enabled: bool,
}

// --- 認証エラーレスポンス構造体 ---
#[derive(Serialize)]
struct AuthError {
    error: String,
    message: String,
}

// --- JSON設定ファイルの構造体 ---
#[derive(Deserialize, Debug, Clone)]
struct McpProcessConfig {
    command: String,
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    build_command: Option<String>,
}

type McpServersConfig = HashMap<String, McpProcessConfig>;

// --- GitHub repository clone function ---
async fn clone_and_build_repository(
    config: &McpProcessConfig,
    work_dir: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let repo_url = config.repository.as_ref().ok_or("Repository is None")?;
    
    println!("[DEBUG] Cloning repository: {}", repo_url);
    
    // Extract repository name
    let repo_name = repo_url
        .split('/')
        .last()
        .ok_or("Invalid repository URL")?;
    
    let clone_path = format!("{}/{}", work_dir, repo_name);
    
    // Remove existing directory if it exists
    if tokio::fs::metadata(&clone_path).await.is_ok() {
        println!("[DEBUG] Removing existing directory: {}", clone_path);
        tokio::fs::remove_dir_all(&clone_path).await.map_err(|e| {
            format!("Failed to remove existing directory: {}", e)
        })?;
    }
    
    // Execute git clone
    let clone_output = Command::new("git")
        .args(["clone", repo_url, &clone_path])
        .output()
        .await
        .map_err(|e| format!("Failed to execute git clone command: {}", e))?;
    
    if !clone_output.status.success() {
        let error_msg = String::from_utf8_lossy(&clone_output.stderr);
        return Err(format!("Git clone failed: {}", error_msg).into());
    }
    
    println!("[DEBUG] Repository cloned to: {}", clone_path);
    
    // Execute build command if specified
    if let Some(build_cmd) = &config.build_command {
        println!("[DEBUG] Executing build command: {}", build_cmd);
        
        let mut build_command = Command::new("sh");
        build_command.args(["-c", build_cmd]);
        build_command.current_dir(&clone_path);
        
        // Add environment variables from config file
        build_command.envs(&config.env);
        
        // Inherit parent environment variables (from Docker container)
        for (key, value) in std::env::vars() {
            build_command.env(key, value);
        }
        
        let build_output = build_command
            .output()
            .await
            .map_err(|e| format!("Failed to execute build command: {}", e))?;
        
        if !build_output.status.success() {
            let error_msg = String::from_utf8_lossy(&build_output.stderr);
            return Err(format!("Build failed: {}", error_msg).into());
        }
        
        println!("[DEBUG] Build completed successfully");
    }
    
    Ok(clone_path)
}

// --- MCPプロセスとの通信用構造体 ---
struct McpServerProcess {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl McpServerProcess {
    async fn query(&mut self, request: &McpRequest) -> Result<McpResponse, String> {
        let start_time = Instant::now();
        println!("[DEBUG] Starting MCP query at {:?}", start_time);
        println!("[DEBUG] Request payload: {:?}", request);

        let request_json = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        println!("[DEBUG] Serialized request: {}", request_json);

        // MCPサーバーには JSON.stringify された文字列を展開して送信
        let mcp_message = &request.command;
        println!("[DEBUG] Sending to MCP server: {}", mcp_message);

        // MCPサーバーに送信
        self.stdin
            .write_all((mcp_message.to_string() + "\n").as_bytes())
            .await
            .map_err(|e| format!("Failed to write to MCP stdin: {}", e))?;

        self.stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush MCP stdin: {}", e))?;

        println!("[DEBUG] Data sent to MCP server, waiting for response...");

        // タイムアウト付きでレスポンスを読み取り
        let response_result = timeout(Duration::from_secs(30), async {
            let mut response_line = String::new();
            match self.stdout.read_line(&mut response_line).await {
                Ok(0) => {
                    println!("[DEBUG] MCP server closed connection (EOF)");
                    Err("MCP server closed the connection (EOF).".to_string())
                }
                Ok(bytes_read) => {
                    println!("[DEBUG] Read {} bytes from MCP server", bytes_read);
                    println!("[DEBUG] Raw response: '{}'", response_line.trim());

                    if response_line.trim().is_empty() {
                        return Err("MCP server returned an empty line.".to_string());
                    }

                    // レスポンスを文字列として返す（再度JSON化はしない）
                    Ok(McpResponse {
                        result: response_line.trim().to_string(),
                    })
                }
                Err(e) => {
                    println!("[DEBUG] Error reading from MCP stdout: {}", e);
                    Err(format!("Failed to read from MCP stdout: {}", e))
                }
            }
        })
        .await;

        match response_result {
            Ok(result) => {
                let elapsed = start_time.elapsed();
                println!("[DEBUG] MCP query completed in {:?}", elapsed);
                result
            }
            Err(_) => {
                println!("[DEBUG] MCP query timed out after 30 seconds");
                Err("MCP server response timeout (30 seconds)".to_string())
            }
        }
    }
}

// --- リクエスト・レスポンスデータ構造 ---
#[derive(Serialize, Deserialize, Debug)]
struct McpRequest {
    command: String,
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
    println!("[DEBUG] Reading config file: {}", config_file_path);

    let config_content = tokio::fs::read_to_string(config_file_path)
        .await
        .map_err(|e| {
            format!(
                "Failed to read MCP config file '{}': {}",
                config_file_path, e
            )
        })?;

    println!("[DEBUG] Config content: {}", config_content);

    let all_configs: McpServersConfig = serde_json::from_str(&config_content).map_err(|e| {
        format!(
            "Failed to parse MCP config file '{}': {}",
            config_file_path, e
        )
    })?;

    println!("[DEBUG] Parsed configs: {:?}", all_configs);

    let server_config = all_configs.get(server_key).ok_or_else(|| {
        format!(
            "MCP server configuration not found for key '{}' in file '{}'",
            server_key, config_file_path
        )
    })?;

    // Handle GitHub repository if specified
    let mut working_directory = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?
        .to_string_lossy()
        .to_string();
    
    if server_config.repository.is_some() {
        println!("[DEBUG] Repository specified, cloning and building...");
        // Create work directory
        tokio::fs::create_dir_all("/tmp/mcp-servers")
            .await
            .map_err(|e| format!("Failed to create work directory: {}", e))?;
        working_directory = clone_and_build_repository(server_config, "/tmp/mcp-servers").await?;
    }

    println!(
        "[DEBUG] Starting MCP server (key: '{}') with command: '{}', args: {:?}, env: {:?}, working_dir: '{}'",
        server_key, &server_config.command, &server_config.args, &server_config.env, working_directory
    );

    let mut command_builder = Command::new(&server_config.command);
    command_builder.args(&server_config.args);
    
    // Add environment variables from config file
    command_builder.envs(&server_config.env);
    
    // Inherit parent environment variables (from Docker container)
    // This allows Docker environment variables to be passed to the MCP server
    for (key, value) in env::vars() {
        command_builder.env(key, value);
    }
    
    command_builder.current_dir(&working_directory);

    command_builder
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    println!("[DEBUG] Spawning MCP process...");
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

    println!("[DEBUG] MCP process spawned successfully, setting up stderr monitoring...");

    let server_key_clone_for_stderr = server_key.to_string();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!(
                        "[MCP Server stderr - {}]: EOF, task finishing.",
                        server_key_clone_for_stderr
                    );
                    break;
                }
                Ok(_) => {
                    print!(
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

    println!("[DEBUG] MCP server setup complete");

    Ok(McpServerProcess {
        stdin,
        stdout: BufReader::new(stdout),
    })
}

// --- Bearer認証ミドルウェア ---
async fn bearer_auth_middleware(
    State(auth_config): State<AuthConfig>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // 認証が無効化されている場合はスキップ
    if !auth_config.enabled {
        return Ok(next.run(request).await);
    }

    // APIキーが設定されていない場合はスキップ
    let expected_api_key = match &auth_config.api_key {
        Some(key) => key,
        None => return Ok(next.run(request).await),
    };

    // Authorizationヘッダーを取得
    let auth_header = match headers.get("authorization") {
        Some(header) => match header.to_str() {
            Ok(header_str) => header_str,
            Err(_) => {
                println!("[DEBUG] Invalid Authorization header format");
                let error_response = AuthError {
                    error: "Unauthorized".to_string(),
                    message: "Invalid Authorization header format".to_string(),
                };
                return Err((StatusCode::UNAUTHORIZED, AxumJson(error_response)));
            }
        },
        None => {
            println!("[DEBUG] Missing Authorization header");
            let error_response = AuthError {
                error: "Unauthorized".to_string(),
                message: "Missing Authorization header".to_string(),
            };
            return Err((StatusCode::UNAUTHORIZED, AxumJson(error_response)));
        }
    };

    // Bearer tokenを抽出
    if !auth_header.starts_with("Bearer ") {
        println!("[DEBUG] Authorization header does not start with 'Bearer '");
        let error_response = AuthError {
            error: "Unauthorized".to_string(),
            message: "Authorization header must use Bearer token".to_string(),
        };
        return Err((StatusCode::UNAUTHORIZED, AxumJson(error_response)));
    }

    let provided_token = &auth_header[7..]; // "Bearer "の7文字をスキップ

    // APIキーを比較
    if provided_token != expected_api_key {
        println!(
            "[DEBUG] Invalid API key provided (length: {})",
            provided_token.len()
        );
        let error_response = AuthError {
            error: "Unauthorized".to_string(),
            message: "Invalid API key".to_string(),
        };
        return Err((StatusCode::UNAUTHORIZED, AxumJson(error_response)));
    }

    println!("[DEBUG] Authentication successful");
    Ok(next.run(request).await)
}

// --- Axum リクエストハンドラ ---
async fn handle_mcp_request_shared(
    State(mcp_process_mutex): State<Arc<Mutex<McpServerProcess>>>,
    AxumJson(payload): AxumJson<McpRequest>,
) -> Result<AxumJson<McpResponse>, StatusCode> {
    println!("[DEBUG] Received HTTP request: {:?}", payload);

    let mut mcp_process_guard = mcp_process_mutex.lock().await;
    println!("[DEBUG] Acquired MCP process mutex lock");

    match mcp_process_guard.query(&payload).await {
        Ok(response) => {
            println!("[DEBUG] MCP query successful: {:?}", response);
            Ok(AxumJson(response))
        }
        Err(e) => {
            eprintln!("[ERROR] MCP query failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// --- 認証設定を作成する関数 ---
fn create_auth_config() -> AuthConfig {
    let api_key = env::var("HTTP_API_KEY").ok();
    let disable_auth = env::var("DISABLE_AUTH")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let enabled = !disable_auth && api_key.is_some();

    if let Some(ref key) = api_key {
        println!(
            "[DEBUG] HTTP API Key configured (length: {} chars)",
            key.len()
        );
    } else {
        println!("[DEBUG] No HTTP API Key configured (HTTP_API_KEY not set)");
    }

    if disable_auth {
        println!("[DEBUG] Authentication disabled by DISABLE_AUTH=true");
    }

    println!("[DEBUG] Authentication enabled: {}", enabled);

    AuthConfig { api_key, enabled }
}

// --- main関数 ---
#[tokio::main]
async fn main() {
    println!("[DEBUG] Starting MCP HTTP server...");

    // 認証設定を作成
    let auth_config = create_auth_config();

    let config_file =
        env::var("MCP_CONFIG_FILE").unwrap_or_else(|_| "mcp_servers.config.json".to_string());
    let mcp_server_key_to_use =
        env::var("MCP_SERVER_NAME").unwrap_or_else(|_| "redmine".to_string());

    println!(
        "[DEBUG] Config file: '{}', Server key: '{}'",
        config_file, mcp_server_key_to_use
    );

    let mcp_server_process_mutex =
        match start_mcp_server_from_config(&config_file, &mcp_server_key_to_use).await {
            Ok(process) => {
                println!("[DEBUG] MCP server started successfully");
                Arc::new(Mutex::new(process))
            }
            Err(e) => {
                eprintln!("[FATAL] Failed to start MCP server process: {}", e);
                eprintln!("Please ensure:");
                eprintln!("1. Node.js is installed and required tools are available");
                eprintln!("2. Git is installed for repository cloning");
                eprintln!("3. Network connectivity is available");
                eprintln!("4. The specified MCP server repository is accessible");
                return;
            }
        };

    let app = Router::new()
        .route("/api/v1", post(handle_mcp_request_shared))
        .layer(middleware::from_fn_with_state(
            auth_config.clone(),
            bearer_auth_middleware,
        ))
        .with_state(mcp_server_process_mutex);

    // Renderの要件に合わせてホストとポートを設定
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener_addr = format!("0.0.0.0:{}", port);

    println!("[DEBUG] Attempting to bind to: {}", listener_addr);

    match tokio::net::TcpListener::bind(&listener_addr).await {
        Ok(listener) => {
            println!(
                "[DEBUG] HTTP server listening on http://{}",
                listener.local_addr().unwrap() // ここでは実際のローカルアドレスを表示
            );
            println!("[DEBUG] Render will forward requests to this port from the public internet.");
            println!("[DEBUG] Ready to accept requests at POST /api/v1");

            if auth_config.enabled {
                println!(
                    "[DEBUG] Authentication is ENABLED - Authorization: Bearer <token> required"
                );
            } else {
                println!("[DEBUG] Authentication is DISABLED - no authorization required");
            }

            if let Err(e) = axum::serve(listener, app.into_make_service()).await {
                eprintln!("[ERROR] Server error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to bind to address {}: {}", listener_addr, e);
        }
    }
}
