use anyhow::Result;
use rpc_router::{Router, Request, Error, CallResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, error};
use std::collections::HashMap;
use std::path::PathBuf;
use petgraph::{Graph, Directed};
use petgraph::graph::{NodeIndex, EdgeIndex};

mod mcp;
use mcp::{types::*, utilities::*};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddEntityRequest {
    pub id: String,
    pub label: String,
    pub properties: Option<HashMap<String, Value>>,
    pub entity_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRelationshipRequest {
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: String,
    pub properties: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryGraphRequest {
    pub pattern: String,
    pub limit: Option<u32>,
    pub filters: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindPathsRequest {
    pub from_entity: String,
    pub to_entity: String,
    pub max_depth: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetNeighborsRequest {
    pub entity_id: String,
    pub depth: Option<u32>,
    pub relationship_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub label: String,
    pub entity_type: Option<String>,
    pub properties: HashMap<String, Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: String,
    pub properties: HashMap<String, Value>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub total_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PathResult {
    pub paths: Vec<Vec<String>>,
    pub path_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStats {
    pub entity_count: u32,
    pub relationship_count: u32,
    pub entity_types: HashMap<String, u32>,
    pub relationship_types: HashMap<String, u32>,
}

pub struct McpKgServer {
    data_path: PathBuf,
    graph: Graph<Entity, Relationship, Directed>,
    entity_index: HashMap<String, NodeIndex>,
}

impl McpKgServer {
    pub async fn new(data_path: PathBuf) -> Result<Self> {
        // Create data directory if it doesn't exist
        tokio::fs::create_dir_all(&data_path).await?;
        
        let graph = Graph::new();
        let entity_index = HashMap::new();
        
        Ok(Self {
            data_path,
            graph,
            entity_index,
        })
    }

    pub async fn serve(&self) -> Result<()> {
        let mut router = Router::new();

        // Standard MCP methods
        router.insert("initialize", initialize);
        router.insert("ping", ping);
        router.insert("logging/setLevel", logging_set_level);
        router.insert("roots/list", roots_list);

        // Tools
        router.insert("tools/list", list_tools);
        router.insert("add_entity", add_entity);
        router.insert("add_relationship", add_relationship);
        router.insert("query_graph", query_graph);
        router.insert("find_paths", find_paths);
        router.insert("get_neighbors", get_neighbors);

        // Resources
        router.insert("resources/list", list_resources);
        router.insert("resources/read", read_resource);

        serve_stdio(router).await
    }
}

async fn list_tools(_: Option<Value>) -> Result<Value, Error> {
    Ok(json!({
        "tools": [
            {
                "name": "add_entity",
                "description": "Add a new entity to the knowledge graph",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Unique identifier for the entity"
                        },
                        "label": {
                            "type": "string",
                            "description": "Human-readable label for the entity"
                        },
                        "entity_type": {
                            "type": "string",
                            "description": "Type/category of the entity (e.g., 'person', 'organization', 'concept')"
                        },
                        "properties": {
                            "type": "object",
                            "description": "Additional properties as key-value pairs",
                            "additionalProperties": true
                        }
                    },
                    "required": ["id", "label"]
                }
            },
            {
                "name": "add_relationship",
                "description": "Add a relationship between two entities in the knowledge graph",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from_entity": {
                            "type": "string",
                            "description": "ID of the source entity"
                        },
                        "to_entity": {
                            "type": "string",
                            "description": "ID of the target entity"
                        },
                        "relationship_type": {
                            "type": "string",
                            "description": "Type of relationship (e.g., 'knows', 'works_for', 'related_to')"
                        },
                        "properties": {
                            "type": "object",
                            "description": "Additional relationship properties",
                            "additionalProperties": true
                        }
                    },
                    "required": ["from_entity", "to_entity", "relationship_type"]
                }
            },
            {
                "name": "query_graph",
                "description": "Query the knowledge graph with patterns and filters",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Query pattern (e.g., entity type, relationship pattern)"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 50)",
                            "default": 50,
                            "minimum": 1,
                            "maximum": 1000
                        },
                        "filters": {
                            "type": "object",
                            "description": "Additional filters to apply",
                            "additionalProperties": true
                        }
                    },
                    "required": ["pattern"]
                }
            },
            {
                "name": "find_paths",
                "description": "Find paths between two entities in the knowledge graph",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from_entity": {
                            "type": "string",
                            "description": "ID of the starting entity"
                        },
                        "to_entity": {
                            "type": "string",
                            "description": "ID of the target entity"
                        },
                        "max_depth": {
                            "type": "integer",
                            "description": "Maximum path depth to search (default: 5)",
                            "default": 5,
                            "minimum": 1,
                            "maximum": 10
                        }
                    },
                    "required": ["from_entity", "to_entity"]
                }
            },
            {
                "name": "get_neighbors",
                "description": "Get neighboring entities connected to a specific entity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "ID of the entity to find neighbors for"
                        },
                        "depth": {
                            "type": "integer",
                            "description": "Depth of neighbors to retrieve (default: 1)",
                            "default": 1,
                            "minimum": 1,
                            "maximum": 3
                        },
                        "relationship_types": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Filter by specific relationship types"
                        }
                    },
                    "required": ["entity_id"]
                }
            }
        ]
    }))
}

async fn add_entity(request: Request) -> Result<CallResponse, Error> {
    let params: AddEntityRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Adding entity: {} ({})", params.id, params.label);

    // Mock implementation - replace with actual graph storage
    let entity = Entity {
        id: params.id.clone(),
        label: params.label,
        entity_type: params.entity_type,
        properties: params.properties.unwrap_or_default(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": format!("Entity '{}' added successfully", params.id)
        }]
    })))
}

async fn add_relationship(request: Request) -> Result<CallResponse, Error> {
    let params: AddRelationshipRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Adding relationship: {} -[{}]-> {}", params.from_entity, params.relationship_type, params.to_entity);

    // Mock implementation - replace with actual graph storage
    let relationship = Relationship {
        id: uuid::Uuid::new_v4().to_string(),
        from_entity: params.from_entity.clone(),
        to_entity: params.to_entity.clone(),
        relationship_type: params.relationship_type.clone(),
        properties: params.properties.unwrap_or_default(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": format!("Relationship '{}' -[{}]-> '{}' added successfully", 
                params.from_entity, params.relationship_type, params.to_entity)
        }]
    })))
}

async fn query_graph(request: Request) -> Result<CallResponse, Error> {
    let params: QueryGraphRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Querying graph with pattern: {}", params.pattern);

    // Mock implementation - replace with actual graph query
    let result = QueryResult {
        entities: vec![],
        relationships: vec![],
        total_count: 0,
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn find_paths(request: Request) -> Result<CallResponse, Error> {
    let params: FindPathsRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Finding paths from {} to {}", params.from_entity, params.to_entity);

    // Mock implementation - replace with actual pathfinding
    let result = PathResult {
        paths: vec![],
        path_count: 0,
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn get_neighbors(request: Request) -> Result<CallResponse, Error> {
    let params: GetNeighborsRequest = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|e| Error::InvalidRequest(format!("Invalid parameters: {}", e)))?;

    info!("Getting neighbors for entity: {}", params.entity_id);

    // Mock implementation - replace with actual neighbor finding
    let result = QueryResult {
        entities: vec![],
        relationships: vec![],
        total_count: 0,
    };

    Ok(CallResponse::from_value(json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&result).unwrap()
        }]
    })))
}

async fn list_resources(_: Option<Value>) -> Result<Value, Error> {
    Ok(json!({
        "resources": [
            {
                "uri": "kg://entity/{id}",
                "name": "Knowledge graph entity",
                "description": "Individual entity data from the knowledge graph",
                "mimeType": "application/json"
            },
            {
                "uri": "kg://graph/stats",
                "name": "Knowledge graph statistics",
                "description": "Overall statistics and metadata about the knowledge graph",
                "mimeType": "application/json"
            }
        ]
    }))
}

async fn read_resource(request: Request) -> Result<CallResponse, Error> {
    // Mock implementation - replace with actual resource reading
    Ok(CallResponse::from_value(json!({
        "contents": [{
            "uri": "kg://graph/stats",
            "mimeType": "application/json",
            "text": json!({
                "entity_count": 0,
                "relationship_count": 0,
                "entity_types": {},
                "relationship_types": {}
            }).to_string()
        }]
    })))
}