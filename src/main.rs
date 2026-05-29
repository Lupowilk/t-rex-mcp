use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router, schemars,
    service::RequestContext,
    task_handler,
    task_manager::{OperationProcessor, OperationResultTransport},
    tool, tool_handler, tool_router,
};



fn main() {
    println!("Hello, world!");
}

#[derive(Clone)]
pub struct TRexServer {
    tool_router: ToolRouter<TRexServer>,
}
