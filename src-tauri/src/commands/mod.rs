mod agent;
mod conversation;
mod files;
mod ollama;
mod plan;

#[allow(dead_code)]
pub use agent::{
    create_agent, delete_agent, export_agent, get_agent, import_agent, list_agents, load_user_agents, seed_starter_agents, update_agent,
};
#[allow(dead_code)]
pub use conversation::{
    create_conversation, get_conversation, list_conversations, load_messages, save_message,
};
#[allow(dead_code)]
pub use ollama::{
    ollama_chat, ollama_chat_stream, ollama_get_default_model, ollama_list_models,
    ollama_pull_model, ollama_set_default_model, ollama_get_provider, ollama_set_provider,
    ollama_get_api_key, ollama_set_api_key,
};
#[allow(dead_code)]
pub use plan::{save_plan_version, list_plan_versions, restore_plan_version};
pub use files::{write_file, read_file};
#[allow(dead_code)]
pub use agent::{Agent, AgentListItem, CreateAgentInput, UpdateAgentInput};
#[allow(dead_code)]
pub use conversation::{Conversation, Message as DbMessage, SaveMessageInput};
#[allow(dead_code)]
pub use ollama::{OllamaChatInput, OllamaChatOutput, OllamaMessage, OllamaModel, LlmProvider};
#[allow(dead_code)]
pub use plan::{PlanVersion, SavePlanVersionInput, RestorePlanVersionInput};