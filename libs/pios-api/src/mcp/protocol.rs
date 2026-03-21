//! Model Context Protocol (MCP) Integration
//! Standardizes integration between AI agents/systems and external data sources or tools.

use alloc::string::String;
use alloc::vec::Vec;

/// The type of an MCP message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpMessageType {
    /// A request to a server or client
    Request,
    /// A response to a request
    Response,
    /// A notification (no response expected)
    Notification,
}

/// A standard MCP message following JSON-RPC 2.0 structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpMessage {
    /// JSON-RPC version, usually "2.0"
    pub jsonrpc: String,
    /// Message ID, for matching requests to responses
    pub id: Option<String>,
    /// Method name for requests and notifications
    pub method: Option<String>,
    /// Parameters for requests and notifications (JSON encoded)
    pub params: Option<String>,
    /// Result for responses (JSON encoded)
    pub result: Option<String>,
    /// Error for failed responses
    pub error: Option<McpError>,
}

/// An MCP Error structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<String>,
}

/// Represents a Tool that can be invoked via MCP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    /// JSON schema describing the input parameters
    pub input_schema: String,
}

/// Represents a Resource that can be accessed via MCP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Represents a Prompt template available via MCP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<McpPromptArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_mcp_message_creation() {
        let error = McpError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };

        let msg = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some("1".to_string()),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        };

        assert_eq!(msg.jsonrpc, "2.0");
        assert_eq!(msg.id, Some("1".to_string()));
        assert!(msg.error.is_some());
        assert_eq!(msg.error.unwrap().code, -32600);
    }

    #[test]
    fn test_mcp_tool_creation() {
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: "{}".to_string(),
        };

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.input_schema, "{}");
    }

    #[test]
    fn test_mcp_resource_creation() {
        let resource = McpResource {
            uri: "file:///test.txt".to_string(),
            name: "Test File".to_string(),
            description: Some("A test resource".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        assert_eq!(resource.uri, "file:///test.txt");
        assert_eq!(resource.name, "Test File");
        assert_eq!(resource.description, Some("A test resource".to_string()));
        assert_eq!(resource.mime_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_mcp_prompt_creation() {
        let prompt = McpPrompt {
            name: "test_prompt".to_string(),
            description: Some("A test prompt".to_string()),
            arguments: vec![
                McpPromptArgument {
                    name: "arg1".to_string(),
                    description: Some("Argument 1".to_string()),
                    required: true,
                }
            ],
        };

        assert_eq!(prompt.name, "test_prompt");
        assert_eq!(prompt.description, Some("A test prompt".to_string()));
        assert_eq!(prompt.arguments.len(), 1);
        assert_eq!(prompt.arguments[0].name, "arg1");
        assert_eq!(prompt.arguments[0].required, true);
    }
}
