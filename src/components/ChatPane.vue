<script setup lang="ts">
import { ref } from 'vue';
import { PropType } from 'vue';
import type { AgentListItem, Message, TemplateEntry, TemplateSource, InstalledTemplate } from '@/types/agent';
import AgentLibrary from '@/components/AgentLibrary.vue';

defineProps({
  messages: { type: Array as PropType<Message[]>, required: true },
  streaming: { type: Boolean, required: true },
  streamingMsgId: { type: String, required: true },
  currentAgentName: { type: String, required: false, default: '' },
  interviewing: { type: Boolean, required: false, default: false },
  buildError: { type: [String] as PropType<string | null>, required: false, default: null },
  modelWarmedUp: { type: Boolean, required: false, default: true },
  agentsProp: { type: Array as PropType<AgentListItem[]>, required: false, default: () => [] },
  templateSourcesProp: { type: Array as PropType<TemplateSource[]>, required: false, default: () => [] },
  installedTemplatesProp: { type: Array as PropType<InstalledTemplate[]>, required: false, default: () => [] },
  marketplaceTemplatesProp: { type: Array as PropType<TemplateEntry[]>, required: false, default: () => [] },
  marketplaceLoadingProp: { type: Boolean as PropType<boolean>, required: false, default: false },
  selectedSourceIdProp: { type: [String, null] as PropType<string | null>, required: false, default: null },
});

const emit = defineEmits([
  'send', 'build', 'retry', 'new-conversation',
  'select-agent', 'create-agent', 'rename-agent', 'duplicate-agent',
  'export-agent', 'delete-agent',
  'add-template-source', 'load-template-sources',
  'load-marketplace-templates', 'install-marketplace-template',
  'load-installed-templates',
]);
const newMessage = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);

function adjustHeight() {
  if (!textareaRef.value) return;
  textareaRef.value.style.height = 'auto';
  textareaRef.value.style.height = `${Math.min(textareaRef.value.scrollHeight, 120)}px`;
}

async function handleSend() {
  if (!newMessage.value.trim()) return;
  emit('send', newMessage.value);
  newMessage.value = '';
  if (textareaRef.value) {
    textareaRef.value.style.height = 'auto';
  }
}

async function handleBuild() {
  emit('build');
}

async function handleRetry() {
  emit('retry');
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleSend();
  }
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}
</script>

<template>
  <div class="chat-pane">
    <div v-if="!modelWarmedUp" class="warming-up">
      <div class="warming-spinner"></div>
      <p>Oppy is warming up. Please wait a moment.</p>
    </div>

    <div class="chat-header">
      <div class="chat-title">
        <div class="chat-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/>
            <path d="M19 10v2a7 7 0 0 1-7 7h-4a7 7 0 0 1-7-7V5a7 7 0 0 1 7-7h4a7 7 0 0 1 7 7Z"/>
          </svg>
        </div>
        <div>
          <h2>{{ currentAgentName || 'Oppy' }}</h2>
          <div class="chat-subtitle">
            <template v-if="interviewing">Interview mode — Oppy is asking questions</template>
            <template v-else-if="streaming">Building agent...</template>
            <template v-else-if="buildError">Build failed — check error or retry</template>
            <template v-else>Agent built — plan ready in panel</template>
          </div>
        </div>
      </div>
      <div class="chat-actions">
        <button v-if="currentAgentName" class="btn-new-chat" @click="emit('new-conversation')" title="Start a fresh conversation">New Chat</button>
        <button v-if="interviewing && modelWarmedUp" class="btn-primary" @click="handleBuild">Build Agent</button>
        <button v-if="interviewing && !modelWarmedUp" class="btn-primary" disabled>Warming up…</button>
        <button v-if="!interviewing && buildError" class="btn-retry" @click="handleRetry">Retry</button>
        <span v-if="!interviewing && buildError" class="build-error">{{ buildError }}</span>
      </div>
    </div>

    <div class="agent-library" v-if="!currentAgentName">
      <AgentLibrary
        :agents="agentsProp"
        :template-sources="templateSourcesProp"
        :installed-templates="installedTemplatesProp"
        :marketplace-templates="marketplaceTemplatesProp"
        :marketplace-loading="marketplaceLoadingProp"
        :selected-source-id="selectedSourceIdProp"
        @select="emit('select-agent')"
        @create="emit('create-agent')"
        @rename="emit('rename-agent')"
        @duplicate="emit('duplicate-agent')"
        @export="emit('export-agent')"
        @delete="emit('delete-agent')"
        @add-template-source="emit('add-template-source')"
        @load-template-sources="emit('load-template-sources')"
        @load-marketplace-templates="emit('load-marketplace-templates')"
        @install-marketplace-template="emit('install-marketplace-template')"
        @load-installed-templates="emit('load-installed-templates')"
      />
    </div>

    <div class="chat-messages" v-if="currentAgentName">
      <div
        v-for="msg of messages"
        :key="msg.id"
        class="message"
        :class="[msg.role, { 'is-streaming': msg.id === streamingMsgId && streaming }]"
      >
        <div class="message-avatar" :class="msg.role">
          {{ msg.role === 'user' ? 'U' : 'O' }}
        </div>
        <div class="message-content">
          <div class="message-text">{{ msg.content }}</div>
          <div v-if="msg.turnPlanJson" class="message-plan-card">
            <div class="plan-card-header">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="12 2 2 7 12 12 22 7 12 2"/>
                <polyline points="2 17 12 22 22 17"/>
                <polyline points="2 12 12 17 22 12"/>
              </svg>
              <span class="plan-card-title">Agent Plan</span>
            </div>
            <div v-if="msg.turnPlanJson.goal" class="plan-card-goal">
              <strong>Goal:</strong> {{ msg.turnPlanJson.goal }}
            </div>
            <div v-if="msg.turnPlanJson.steps && msg.turnPlanJson.steps.length" class="plan-card-steps">
              <strong>Steps ({{ msg.turnPlanJson.steps.length }}):</strong>
              <ol class="plan-steps-preview">
                <li v-for="(step, idx) in msg.turnPlanJson.steps.slice(0, 3)" :key="idx">{{ step.action }}</li>
                <li v-if="msg.turnPlanJson.steps.length > 3" class="more-steps">
                  + {{ msg.turnPlanJson.steps.length - 3 }} more steps (see Plan panel)
                </li>
              </ol>
            </div>
          </div>
          <div class="message-meta">
            <span class="message-role">{{ msg.role === 'user' ? 'You' : 'Oppy' }}</span>
            <span class="message-time">{{ formatTime(msg.createdAt) }}</span>
            <span
              v-if="streaming && msg.id === streamingMsgId"
              class="streaming-indicator"
              role="status"
              aria-label="Oppy is responding"
            >
              Oppy
              <span class="typing-dots" aria-hidden="true"><i></i><i></i><i></i></span>
            </span>
          </div>
        </div>
      </div>
    </div>

    <div class="chat-input-area" v-if="currentAgentName && modelWarmedUp">
      <textarea
        ref="textareaRef"
        v-model="newMessage"
        placeholder="Tell Oppy what you want your agent to do…"
        rows="1"
        @keydown="handleKeyDown"
        @input="adjustHeight"
        class="chat-textarea"
      ></textarea>
      <button class="btn-send" @click="handleSend" :disabled="streaming || !newMessage.trim()">Send</button>
    </div>
    <div v-if="currentAgentName && !modelWarmedUp" class="chat-input-disabled">
      <span class="disabled-label">Input disabled while model is warming up.</span>
    </div>
  </div>
</template>

<style scoped>
.chat-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--surface-card);
  border-right: 1px solid var(--border);
  overflow-y: auto;
  position: relative;
}

.chat-header {
  padding: var(--space-4) var(--space-6);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;
  background: var(--surface-hover);
}

.chat-icon {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  background: var(--primary-bg);
  color: var(--primary);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.chat-title h2 {
  margin: 0;
  font-size: var(--type-lg);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.chat-subtitle {
  font-size: var(--type-xs);
  color: var(--text-secondary);
  margin-top: var(--space-1);
}

.chat-actions {
  margin-left: auto;
  display: flex;
  gap: var(--space-2);
  align-items: center;
}

.btn-new-chat {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  transition: background 0.2s, border-color 0.2s;
}

.btn-new-chat:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-build {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--primary);
  color: var(--primary-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s, transform 0.1s;
}

.btn-build:hover:not(:disabled) {
  background: var(--primary-hover);
  border-color: var(--border-focus);
}

.btn-build:active:not(:disabled) {
  transform: translateY(1px);
}

.btn-build:disabled {
  background: var(--surface-muted);
  color: var(--text-muted);
  cursor: not-allowed;
}

.btn-retry {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s, border-color 0.2s;
}

.btn-retry:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.build-error {
  font-size: var(--type-xs);
  color: var(--danger-fg);
  margin-left: var(--space-2);
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.warming-up {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
  text-align: center;
  padding: var(--space-8);
}

.warming-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: var(--space-3);
}

.warming-up p {
  font-size: var(--type-sm);
}

.agent-library-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 0;
  padding-bottom: 4.5rem;
}

.empty-state {
  text-align: center;
  color: var(--text-muted);
}

.empty-icon {
  width: 48px;
  height: 48px;
  color: var(--text-muted);
  margin: 0 auto var(--space-3);
}

.empty-state p {
  font-size: var(--type-base);
  margin: 0;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.message {
  display: flex;
  gap: var(--space-3);
  max-width: 85%;
  animation: fadeIn 0.2s ease-out;
}

.message.user {
  align-self: flex-end;
  flex-direction: row-reverse;
}

.message.assistant {
  align-self: flex-start;
}

.message-avatar {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.85rem;
  font-weight: 600;
  flex-shrink: 0;
}

.message.user .message-avatar {
  background: var(--primary);
  color: #ffffff;
}

.message.assistant .message-avatar {
  background: var(--surface-muted);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.message-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.message.user .message-content {
  align-items: flex-end;
  text-align: right;
}

.message.assistant .message-content {
  align-items: flex-start;
  text-align: left;
}

.message-text {
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  line-height: 1.5;
  white-space: pre-wrap;
  max-width: 100%;
}

.message.user .message-text {
  background: var(--primary);
  color: #ffffff;
  border-bottom-right-radius: var(--radius-sm);
}

.message.assistant .message-text {
  background: var(--surface-hover);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-bottom-left-radius: var(--radius-sm);
}

.message-meta {
  display: flex;
  gap: var(--space-2);
  font-size: var(--type-xs);
  color: var(--text-muted);
}

.message.user .message-meta {
  justify-content: flex-end;
}

.message.assistant .message-meta {
  justify-content: flex-start;
}

.streaming-indicator {
  color: var(--primary);
  font-size: var(--type-xs);
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.typing-dots {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.typing-dots i {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.4;
  animation: typing-dot 1s infinite ease-in-out;
}

.typing-dots i:nth-child(2) {
  animation-delay: 0.15s;
}

.typing-dots i:nth-child(3) {
  animation-delay: 0.3s;
}

@keyframes typing-dot {
  0%, 60%, 100% { opacity: 0.4; transform: translateY(0); }
  30% { opacity: 1; transform: translateY(-2px); }
}

.chat-input-area {
  padding: var(--space-4);
  border-top: 1px solid var(--border);
  background: var(--surface-hover);
  display: flex;
  gap: var(--space-3);
  align-items: flex-end;
  flex-shrink: 0;
}

.chat-textarea {
  flex: 1;
  min-height: 44px;
  max-height: 120px;
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  resize: vertical;
  font-family: inherit;
  background: var(--surface-card);
  color: var(--text-primary);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.chat-textarea:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(123, 179, 232, 0.1);
}

.btn-send {
  padding: var(--space-2) var(--space-5);
  font-size: var(--type-sm);
  background: var(--primary);
  color: var(--primary-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
  transition: background 0.2s, transform 0.1s;
  white-space: nowrap;
}

.btn-send:hover:not(:disabled) {
  background: var(--primary-hover);
  border-color: var(--border-focus);
}

.btn-send:active:not(:disabled) {
  transform: translateY(1px);
}

.btn-send:disabled {
  background: var(--surface-muted);
  color: var(--text-muted);
  cursor: not-allowed;
}

.chat-input-disabled {
  padding: var(--space-4);
  border-top: 1px solid var(--border);
  background: var(--surface-hover);
  color: var(--text-muted);
  font-size: var(--type-sm);
  text-align: center;
  flex-shrink: 0;
}

.disabled-label {
  display: inline-block;
}

.message-plan-card {
  margin-top: var(--space-3);
  padding: var(--space-3) var(--space-4);
  background: var(--primary-bg);
  border: 1px solid var(--primary);
  border-radius: var(--radius-md);
  font-size: var(--type-sm);
  color: var(--text-primary);
}

.plan-card-header {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-weight: 600;
  color: var(--primary);
  margin-bottom: var(--space-2);
}

.plan-card-goal {
  margin-bottom: var(--space-2);
  line-height: 1.5;
}

.plan-card-steps strong {
  display: block;
  margin-bottom: var(--space-1);
}

.plan-steps-preview {
  margin: 0;
  padding-left: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.more-steps {
  list-style-type: none;
  font-style: italic;
  color: var(--text-secondary);
  margin-top: var(--space-1);
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>