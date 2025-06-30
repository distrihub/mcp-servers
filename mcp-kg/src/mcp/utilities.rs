use crate::mcp::types::*;
use crate::mcp::{JSONRPC_VERSION, PROTOCOL_VERSION, SERVER_NAME, SERVER_VERSION};
use rpc_router::{Router, Request, Error};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use signal_hook::consts::SIGTERM;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::thread;

pub async fn initialize(_: InitializeRequestParams) -> Result<InitializeResult, Error> {
    Ok(InitializeResult {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            prompts: Some(PromptsCapability {
                list_changed: Some(false),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(false),
                list_changed: Some(false),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(false),
            }),
            logging: Some(LoggingCapability {}),
            experimental: None,
        },
        server_info: ServerInfo {
            name: SERVER_NAME.to_string(),
            version: SERVER_VERSION.to_string(),
        },
        instructions: Some("Code analysis, formatting, and development tools server".to_string()),
    })
}

pub async fn ping(_: Option<Value>) -> Result<Value, Error> {
    Ok(json!({}))
}

pub async fn logging_set_level(_: LoggingSetLevelParams) -> Result<Value, Error> {
    Ok(json!({}))
}

pub async fn roots_list(_: Option<Value>) -> Result<RootsListResult, Error> {
    Ok(RootsListResult { roots: vec![] })
}

pub fn notifications_initialized() {
    // Handle initialization notification
}

pub fn notifications_cancelled(_params: CancelledNotification) {
    // Handle cancellation notification
}

pub fn graceful_shutdown() {
    // Cleanup resources before shutdown
}

pub async fn serve_stdio(router: Router) -> anyhow::Result<()> {
    // Signal handling
    let mut signals = Signals::new([SIGTERM, SIGINT]).unwrap();
    thread::spawn(move || {
        for _sig in signals.forever() {
            graceful_shutdown();
            std::process::exit(0);
        }
    });

    // Process JSON-RPC from MCP client
    let mut line = String::new();
    let input = io::stdin();
    let mut logging_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("/tmp/mcp-coder.jsonl")
        .unwrap();

    while input.read_line(&mut line).unwrap() != 0 {
        let line = std::mem::take(&mut line);
        writeln!(logging_file, "{}", line).unwrap();
        
        if !line.is_empty() {
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                // Notifications, no response required
                if json_value.is_object() && json_value.get("id").is_none() {
                    if let Some(method) = json_value.get("method") {
                        if method == "notifications/initialized" {
                            notifications_initialized();
                        } else if method == "notifications/cancelled" {
                            let params_value = json_value.get("params").unwrap();
                            let cancel_params: CancelledNotification =
                                serde_json::from_value(params_value.clone()).unwrap();
                            notifications_cancelled(cancel_params);
                        }
                    }
                } else if let Ok(mut rpc_request) = Request::from_value(json_value) {
                    // Normal JSON-RPC message, response expected
                    let id = rpc_request.id.clone();
                    if rpc_request.method == "tools/call" {
                        let params = serde_json::from_value::<ToolCallRequestParams>(
                            rpc_request.params.unwrap(),
                        )
                        .unwrap();
                        rpc_request = Request {
                            id: id.clone(),
                            method: params.name,
                            params: params.arguments,
                        }
                    }
                    match router.call(rpc_request).await {
                        Ok(call_response) => {
                            if !call_response.value.is_null() {
                                let response =
                                    JsonRpcResponse::new(id, call_response.value.clone());
                                let response_json = serde_json::to_string(&response).unwrap();
                                writeln!(logging_file, "{}\n", response_json).unwrap();
                                println!("{}", response_json);
                            }
                        }
                        Err(error) => match &error.error {
                            Error::Handler(handler) => {
                                if let Some(error_value) = handler.get::<Value>() {
                                    let json_error = json!({
                                        "jsonrpc": "2.0",
                                        "error": error_value,
                                        "id": id
                                    });
                                    let response = serde_json::to_string(&json_error).unwrap();
                                    writeln!(logging_file, "{}\n", response).unwrap();
                                    println!("{}", response);
                                }
                            }
                            _ => {
                                let json_error = JsonRpcError::new(id, -1, "Invalid json-rpc call");
                                let response = serde_json::to_string(&json_error).unwrap();
                                writeln!(logging_file, "{}\n", response).unwrap();
                                println!("{}", response);
                            }
                        },
                    }
                }
            }
        }
    }
    Ok(())
}