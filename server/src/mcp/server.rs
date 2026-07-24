//! MCP server handler.
//!
//! [`RuckChatMcpServer`] implements the `rmcp` [`ServerHandler`] trait. It
//! reads the authenticated [`UserId`] from each request's extensions and
//! delegates tool calls and resource reads to [`McpService`].

use crate::mcp::{resources, tools};
use crate::services::McpService;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorData, Implementation, ListResourcesResult,
    ListToolsResult, ProtocolVersion, ReadResourceRequestParams, ReadResourceResult,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use ruckchat_id::UserId;

/// Shortcut for the HTTP request parts injected by the Streamable HTTP
/// transport, including any custom Axum extensions such as the caller's
/// [`UserId`].
type HttpParts = axum::http::request::Parts;

/// MCP server handler backed by the existing RuckChat service layer.
#[derive(Clone)]
pub struct RuckChatMcpServer {
    mcp: McpService,
}

impl RuckChatMcpServer {
    /// Creates the handler from an [`McpService`].
    #[must_use]
    pub fn new(mcp: McpService) -> Self {
        Self { mcp }
    }
}

impl ServerHandler for RuckChatMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_protocol_version(ProtocolVersion::LATEST)
        .with_server_info(Implementation::new(
            "ruckchat-mcp",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "RuckChat MCP server. Tools and resources are scoped to the authenticated user."
                .to_string(),
        )
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        Ok(ListToolsResult::with_all_items(tools::all_tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let caller_id = caller_id_from_context(&context)
            .ok_or_else(|| ErrorData::invalid_request("missing authenticated user", None))?;
        tools::handle_tool_call(
            &self.mcp,
            caller_id,
            &request.name,
            request.arguments.as_ref(),
        )
        .await
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::ErrorData> {
        Ok(ListResourcesResult::with_all_items(
            resources::all_resources(),
        ))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::ErrorData> {
        let caller_id = caller_id_from_context(&context)
            .ok_or_else(|| ErrorData::invalid_request("missing authenticated user", None))?;
        resources::read_resource(&self.mcp, caller_id, &request.uri).await
    }
}

fn caller_id_from_context(context: &RequestContext<RoleServer>) -> Option<UserId> {
    context
        .extensions
        .get::<HttpParts>()
        .and_then(|parts| parts.extensions.get::<UserId>())
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_info_advertises_tools_and_resources() {
        let server = RuckChatMcpServer::new(crate::services::McpService::new(
            crate::services::McpServiceDeps {
                channels: crate::services::channel::ChannelService::new(
                    crate::services::channel::ChannelServiceDeps {
                        channels: std::sync::Arc::new(crate::testing::MockChannelRepository::new()),
                        channel_memberships: std::sync::Arc::new(
                            crate::testing::MockChannelMembershipRepository::new(),
                        ),
                        memberships: std::sync::Arc::new(
                            crate::testing::MockOrganizationMembershipRepository::new(),
                        ),
                        authorization: crate::services::AuthorizationService::new(),
                    },
                ),
                direct_messages: crate::services::direct_message::DirectMessageService::new(
                    crate::services::direct_message::DirectMessageServiceDeps {
                        conversations: std::sync::Arc::new(
                            crate::testing::MockDirectMessageConversationRepository::new(),
                        ),
                        memberships: std::sync::Arc::new(
                            crate::testing::MockOrganizationMembershipRepository::new(),
                        ),
                    },
                ),
                messages: crate::services::message::MessageService::new(
                    crate::services::message::MessageServiceDeps {
                        messages: std::sync::Arc::new(crate::testing::MockMessageRepository::new()),
                        channels: std::sync::Arc::new(crate::testing::MockChannelRepository::new()),
                        channel_memberships: std::sync::Arc::new(
                            crate::testing::MockChannelMembershipRepository::new(),
                        ),
                        memberships: std::sync::Arc::new(
                            crate::testing::MockOrganizationMembershipRepository::new(),
                        ),
                        conversations: std::sync::Arc::new(
                            crate::testing::MockDirectMessageConversationRepository::new(),
                        ),
                        users: std::sync::Arc::new(crate::testing::MockUserRepository::new()),
                        authorization: crate::services::AuthorizationService::new(),
                        events: std::sync::Arc::new(crate::testing::MockEventBus::new()),
                    },
                ),
                users: crate::services::user::UserService::new(
                    crate::services::user::UserServiceDeps {
                        users: std::sync::Arc::new(crate::testing::MockUserRepository::new()),
                        memberships: std::sync::Arc::new(
                            crate::testing::MockOrganizationMembershipRepository::new(),
                        ),
                    },
                ),
                organizations: crate::services::organization::OrganizationService::new(
                    crate::services::organization::OrganizationServiceDeps {
                        organizations: std::sync::Arc::new(
                            crate::testing::MockOrganizationRepository::new(),
                        ),
                        users: std::sync::Arc::new(crate::testing::MockUserRepository::new()),
                        memberships: std::sync::Arc::new(
                            crate::testing::MockOrganizationMembershipRepository::new(),
                        ),
                        settings: std::sync::Arc::new(
                            crate::testing::MockOrganizationSettingsRepository::new(),
                        ),
                        authorization: crate::services::AuthorizationService::new(),
                    },
                ),
                memberships: std::sync::Arc::new(
                    crate::testing::MockOrganizationMembershipRepository::new(),
                ),
            },
            true,
        ));
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.resources.is_some());
    }
}
