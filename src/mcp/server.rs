use crate::api::ApiClient;
use crate::wiki::WikiClient;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    model::{ServerCapabilities, ServerInfo},
    tool_handler,
};

#[derive(Debug, Clone)]
pub struct Gw3McpServer {
    #[allow(dead_code)]
    pub(crate) tool_router: ToolRouter<Self>,
    pub(crate) api_client: ApiClient,
    pub(crate) wiki_client: WikiClient,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Gw3McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }
}

pub async fn serve_stdio() -> Result<(), Box<dyn std::error::Error>> {
    let service = Gw3McpServer::from_env()?
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
