//! MCP resource definitions and read handlers.
//!
//! Resources use the `ruckchat://` URI scheme and expose metadata for
//! organizations, channels, conversations, and messages that the caller is
//! authorized to see.

use crate::services::McpService;
use rmcp::model::{ErrorData, ReadResourceResult, Resource, ResourceContents};
use ruckchat_id::{ChannelId, MessageId, OrganizationId, UserId};
use serde_json::Value;

/// All static resources exposed by the RuckChat MCP server.
#[must_use]
pub fn all_resources() -> Vec<Resource> {
    vec![
        Resource::new("ruckchat://organization/{id}", "Organization metadata"),
        Resource::new("ruckchat://channel/{id}", "Channel metadata"),
        Resource::new("ruckchat://conversation/{id}", "Channel or DM metadata"),
        Resource::new("ruckchat://message/{id}", "Message content and metadata"),
    ]
}

/// Reads a `ruckchat://` resource.
pub async fn read_resource(
    mcp: &McpService,
    caller_id: UserId,
    uri: &str,
) -> Result<ReadResourceResult, ErrorData> {
    let Some((kind, id_str)) = parse_uri(uri) else {
        return Err(ErrorData::invalid_params(
            format!("invalid ruckchat resource URI: {uri}"),
            None,
        ));
    };

    let id = id_str
        .parse::<uuid::Uuid>()
        .map_err(|err| ErrorData::invalid_params(format!("invalid resource id: {err}"), None))?;

    let value: Value = match kind {
        "organization" => match mcp
            .get_organization(caller_id, OrganizationId::from_uuid(id))
            .await
        {
            Ok(org) => serde_json::to_value(org),
            Err(err) => return Ok(resource_error(err)),
        },
        "channel" => match mcp.get_channel(caller_id, ChannelId::from_uuid(id)).await {
            Ok(channel) => serde_json::to_value(channel),
            Err(err) => return Ok(resource_error(err)),
        },
        "conversation" => match read_conversation(mcp, caller_id, id).await {
            Ok(value) => Ok(value),
            Err(err) => return Ok(resource_error(err)),
        },
        "message" => match mcp.get_message(caller_id, MessageId::from_uuid(id)).await {
            Ok(message) => serde_json::to_value(message),
            Err(err) => return Ok(resource_error(err)),
        },
        _ => {
            return Err(ErrorData::invalid_params(
                format!("unknown resource kind: {kind}"),
                None,
            ));
        }
    }
    .map_err(|err| ErrorData::internal_error(format!("serialization failed: {err}"), None))?;

    let text = serde_json::to_string_pretty(&value)
        .unwrap_or_else(|err| format!("{{\"error\":\"{err}\"}}"));
    Ok(ReadResourceResult::new(vec![
        ResourceContents::TextResourceContents {
            uri: uri.to_string(),
            mime_type: Some("application/json".to_string()),
            text,
            meta: None,
        },
    ]))
}

async fn read_conversation(
    mcp: &McpService,
    caller_id: UserId,
    conversation_id: uuid::Uuid,
) -> Result<Value, ruckchat_common::Error> {
    if let Ok(channel) = mcp
        .get_channel(caller_id, ChannelId::from_uuid(conversation_id))
        .await
    {
        return serde_json::to_value(channel).map_err(|err| {
            ruckchat_common::Error::Internal(format!("serialization failed: {err}"))
        });
    }
    let dm = mcp
        .get_direct_message_conversation(caller_id, conversation_id)
        .await?;
    serde_json::to_value(dm)
        .map_err(|err| ruckchat_common::Error::Internal(format!("serialization failed: {err}")))
}

fn parse_uri(uri: &str) -> Option<(&str, &str)> {
    let rest = uri.strip_prefix("ruckchat://")?;
    let slash = rest.find('/')?;
    let kind = &rest[..slash];
    let id = &rest[slash + 1..];
    if id.is_empty() {
        return None;
    }
    Some((kind, id))
}

fn resource_error(err: ruckchat_common::Error) -> ReadResourceResult {
    ReadResourceResult::new(vec![ResourceContents::TextResourceContents {
        uri: String::new(),
        mime_type: Some("text/plain".to_string()),
        text: err.to_string(),
        meta: None,
    }])
}
