
# CLAUDE IMPLEMENTATION CONTRACT

Never change architecture without updating ADRs.

Implementation order:

1. Cargo workspace
2. Shared crates
3. Database schema
4. Domain layer
5. Services
6. REST API
7. WebSocket server
8. MCP server
9. Plugin SDK
10. Desktop
11. Mobile
12. Migration tools

Every completed feature must include:
- Unit tests
- Integration tests
- OpenAPI updates
- Documentation
