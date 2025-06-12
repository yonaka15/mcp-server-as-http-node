use mcp_server_as_http_core::{
    server::McpHttpServer,
    config::{ServerConfig, RuntimeConfig, NodeConfig},
    auth::AuthConfig,
    runtime::Runtime,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[INFO] Starting MCP HTTP Server for Node.js runtime...");

    // Environment-based configuration
    let config_file = env::var("MCP_CONFIG_FILE")
        .unwrap_or_else(|_| "mcp_servers.config.json".to_string());
    let server_name = env::var("MCP_SERVER_NAME")
        .unwrap_or_else(|_| "redmine".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;

    // Authentication configuration
    let auth_config = AuthConfig {
        api_key: env::var("HTTP_API_KEY").ok(),
        enabled: env::var("DISABLE_AUTH")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .map(|disable| !disable)
            .unwrap_or(true),
    };

    // Node.js optimized runtime configuration
    let node_config = NodeConfig {
        version: ">=18.0.0".to_string(),
        package_manager: env::var("NODE_PACKAGE_MANAGER")
            .unwrap_or_else(|_| "npm".to_string()),
        enable_typescript: env::var("ENABLE_TYPESCRIPT")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
        auto_install_dependencies: env::var("AUTO_INSTALL_DEPS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),
    };

    // Server configuration
    let server_config = ServerConfig {
        config_file,
        server_name,
        runtime_type: Runtime::Node,
        runtime_config: RuntimeConfig::Node(node_config),
        port,
        host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        auth: auth_config,
        work_directory: env::var("WORK_DIR")
            .unwrap_or_else(|_| "/tmp/mcp-servers".to_string()),
    };

    // Start the MCP HTTP server
    let mut server = McpHttpServer::new(server_config).await?;
    
    println!("[INFO] MCP HTTP Server (Node.js) starting on port {}", port);
    
    server.start().await?;

    Ok(())
}
