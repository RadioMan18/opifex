mod agent;
mod conversation;
mod files;
mod ollama;
mod plan;

pub use agent::{
    create_agent, delete_agent, get_agent, list_agents, seed_starter_agents, update_agent,
    export_agent, import_agent, rename_agent, duplicate_agent,
};
pub use conversation::{
    create_conversation, get_conversation, list_conversations, load_messages, save_message,
};
pub use ollama::{
    ollama_chat, ollama_chat_stream, ollama_get_default_model, ollama_list_models,
    ollama_pull_model, ollama_set_default_model, ollama_get_provider, ollama_set_provider,
    ollama_get_api_key, ollama_set_api_key,
};
pub use plan::{save_plan_version, list_plan_versions, restore_plan_version};
pub use files::write_file;
pub use agent::{Agent, AgentListItem, CreateAgentInput, UpdateAgentInput};
pub use conversation::{Conversation, Message as DbMessage, SaveMessageInput};
pub use ollama::{OllamaChatInput, OllamaChatOutput, OllamaMessage, OllamaModel, LlmProvider};
pub use plan::{PlanVersion, SavePlanVersionInput, RestorePlanVersionInput};
