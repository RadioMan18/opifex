mod agent;
mod conversation;
mod connector;
mod files;
mod ollama;
mod plan;
mod run;
mod template;

#[allow(dead_code)]
pub use agent::{
    create_agent, delete_agent, export_agent, get_agent, import_agent, list_agents, load_user_agents, seed_starter_agents, update_agent,
};
#[allow(dead_code)]
pub use conversation::{
    create_conversation, get_conversation, list_conversations, load_messages, save_message,
};
#[allow(dead_code)]
pub use connector::{
    check_tool_approval, create_connector, delete_connector, get_connector, grant_connector,
    list_connector_grants, list_connectors, revoke_connector, update_connector,
};
#[allow(dead_code)]
pub use run::{
    pin_agent_data, run_agent_graph, unpin_agent_data, get_pinned_data, get_run_status,
    create_skill, list_skills,
    create_knowledge_source, list_knowledge_sources,
    create_trigger, list_triggers,
    create_subagent, list_subagents,
    list_abilities, toggle_ability,
    create_channel, list_channels,
    PinDataInput, PinnedDataDto, RunAgentGraphInput, RunStatusDto, UnpinDataInput,
    SubagentDto, AbilityDto, ChannelDto,
    CreateSubagentInput, ListSubagentsInput,
    ListAbilitiesInput,
    CreateChannelInput, ListChannelsInput,
    CreateInstructionChangeInput, InstructionChangeDto, ListInstructionChangesInput,
    CreateEvaluationInput, EvaluationDto, ListEvaluationsInput,
    CreateReflectionInput, ReflectionDto, ListReflectionsInput,
    CreateArtifactInput, ArtifactDto, ListArtifactsInput,
    SetMessagingModeInput, MessagingModeDto, GetMessagingModeInput,
    CreateSetupStepInput, SetupStepDto, ListSetupStepsInput,
    ApplyOppyEditInput, ApplyOppyEditOutput,
};
#[allow(dead_code)]
pub use ollama::{
    ollama_chat, ollama_chat_stream, ollama_get_default_model, ollama_list_models,
    ollama_pull_model, ollama_set_default_model, ollama_get_provider, ollama_set_provider,
    ollama_get_api_key, ollama_set_api_key,
};
#[allow(dead_code)]
pub use plan::{save_plan_version, list_plan_versions, restore_plan_version};
#[allow(dead_code)]
pub use template::{
    add_template_source, fetch_templates, install_template, list_installed_templates,
    list_template_sources, InstalledTemplate, TemplateEntry, TemplateSource,
};
#[allow(dead_code)]
pub use template::{AddTemplateSourceInput, InstallTemplateInput, InstallTemplateResult};
pub use files::{write_file, read_file};
#[allow(dead_code)]
pub use agent::{Agent, AgentListItem, CreateAgentInput, UpdateAgentInput};
#[allow(dead_code)]
pub use conversation::{Conversation, Message as DbMessage, SaveMessageInput};
#[allow(dead_code)]
pub use ollama::{OllamaChatInput, OllamaChatOutput, OllamaMessage, OllamaModel, LlmProvider};
#[allow(dead_code)]
pub use plan::{PlanVersion, SavePlanVersionInput, RestorePlanVersionInput};
#[allow(dead_code)]
pub use connector::{ConnectorBlockDto, ConnectorGrantDto, CreateConnectorInput, UpdateConnectorInput, GrantConnectorInput, ListConnectorsInput, ListGrantsInput};