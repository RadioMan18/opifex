<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import { PropType } from 'vue';
import type { AgentListItem, Conversation, Message, TemplateEntry, TemplateSource, InstalledTemplate } from '@/types/agent';
import { migratePlan } from '@/lib/planSchema';
import AgentLibrary from '@/components/AgentLibrary.vue';

const props = defineProps({
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
  conversationsProp: { type: Array as PropType<Conversation[]>, required: false, default: () => [] },
  transcriptProp: { type: Array as PropType<Message[] | null>, required: false, default: null },
  transcriptTitleProp: { type: String, required: false, default: '' },
  transcriptConversationIdProp: { type: [String, null] as PropType<string | null>, required: false, default: null },
});

const emit = defineEmits([
  'send', 'retry', 're-draft', 'new-conversation',
  'open-transcript', 'close-transcript', 'clear-history',
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
  if (!props.interviewing || !newMessage.value.trim()) return;
  emit('send', newMessage.value);
  newMessage.value = '';
  if (textareaRef.value) {
    textareaRef.value.style.height = 'auto';
  }
}

function handleRetry() {
  emit('retry');
}

// Re-draft clears the plan, so it takes two clicks.
const reDraftArmed = ref(false);
let reDraftTimer: ReturnType<typeof setTimeout> | null = null;

function handleReDraft() {
  if (!reDraftArmed.value) {
    reDraftArmed.value = true;
    if (reDraftTimer) clearTimeout(reDraftTimer);
    reDraftTimer = setTimeout(() => {
      reDraftArmed.value = false;
    }, 4000);
    return;
  }
  if (reDraftTimer) clearTimeout(reDraftTimer);
  reDraftArmed.value = false;
  emit('re-draft');
}

// Clearing history deletes every transcript for the agent, so it takes two clicks.
const clearArmed = ref(false);
let clearTimer: ReturnType<typeof setTimeout> | null = null;

const readOnly = computed(() => props.transcriptProp !== null);
const displayMessages = computed(() => props.transcriptProp ?? props.messages);

function handleHistoryChange(event: Event) {
  const id = (event.target as HTMLSelectElement).value;
  if (!id) {
    emit('close-transcript');
    return;
  }
  emit('open-transcript', id);
}

function handleClearHistory() {
  if (!clearArmed.value) {
    clearArmed.value = true;
    if (clearTimer) clearTimeout(clearTimer);
    clearTimer = setTimeout(() => {
      clearArmed.value = false;
    }, 4000);
    return;
  }
  if (clearTimer) clearTimeout(clearTimer);
  clearArmed.value = false;
  emit('clear-history');
}

/** Chains a turn produced, shown against each turn in the read-only transcript. */
function planChainCount(plan: unknown): number {
  return plan ? migratePlan(plan).chains.length : 0;
}

onBeforeUnmount(() => {
  if (reDraftTimer) clearTimeout(reDraftTimer);
  if (clearTimer) clearTimeout(clearTimer);
});

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
            <template v-if="readOnly">Reading a past intake — read-only</template>
            <template v-else-if="!interviewing">Plan is yours — Oppy is read-only</template>
            <template v-else-if="streaming">Oppy is drafting the plan…</template>
            <template v-else-if="buildError">Draft failed — retry or keep editing in the canvas</template>
            <template v-else>Intake — Oppy is drafting, you own the canvas</template>
          </div>
        </div>
      </div>
      <div class="chat-actions">
        <select
          v-if="currentAgentName && conversationsProp.length"
          class="history-select"
          :value="transcriptConversationIdProp ?? ''"
          :disabled="streaming"
          aria-label="Intake history"
          @change="handleHistoryChange"
        >
          <option value="">Current intake</option>
          <option v-for="conv in conversationsProp" :key="conv.id" :value="conv.id">{{ conv.title }}</option>
        </select>
        <button
          v-if="currentAgentName && conversationsProp.length"
          class="btn-new-chat"
          @click="handleClearHistory"
          title="Deletes every intake transcript for this agent; the plan is not touched"
        >{{ clearArmed ? 'Confirm clear' : 'Clear history' }}</button>
        <button v-if="currentAgentName && interviewing" class="btn-new-chat" @click="emit('new-conversation')" title="Start a fresh intake conversation">New Chat</button>
        <button
          v-if="!interviewing"
          class="btn-new-chat"
          @click="handleReDraft"
          title="This clears the plan and starts a fresh intake"
        >{{ reDraftArmed ? 'Confirm re-draft' : 'Re-draft' }}</button>
        <button v-if="!modelWarmedUp" class="btn-primary" disabled>Warming up…</button>
        <button v-if="interviewing && buildError" class="btn-retry" @click="handleRetry">Retry</button>
        <span v-if="interviewing && buildError" class="build-error">{{ buildError }}</span>
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
        v-for="msg of displayMessages"
        :key="msg.id"
        class="message"
        :class="[msg.role, { 'is-streaming': msg.id === streamingMsgId && streaming }]"
      >
        <div class="message-avatar" :class="msg.role">
          {{ msg.role === 'user' ? 'U' : 'O' }}
        </div>
        <div class="message-content">
          <div class="message-text">{{ msg.content }}</div>
          <div v-if="readOnly && msg.turnPlanJson" class="turn-plan-note">
            Plan after this turn: {{ planChainCount(msg.turnPlanJson) }} chains
          </div>
          <div class="message-meta">
            <span class="message-role">{{ msg.role === 'user' ? 'You' : 'Oppy' }}</span>
            <span class="message-time">{{ formatTime(msg.createdAt) }}</span>
            <span
              v-if="!readOnly && streaming && msg.id === streamingMsgId"
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

    <div v-if="readOnly" class="chat-readonly-bar">
      <span class="disabled-label">“{{ transcriptTitleProp }}” — reading history does not change the plan.</span>
      <button class="btn-retry" @click="emit('close-transcript')">Back to current</button>
    </div>
    <div class="chat-input-area" v-if="currentAgentName && modelWarmedUp && !readOnly">
      <textarea
        ref="textareaRef"
        v-model="newMessage"
        :placeholder="interviewing ? 'Tell Oppy what you want your agent to do…' : 'This plan is yours now — use Re-draft to hand it back to Oppy.'"
        :disabled="!interviewing"
        rows="1"
        @keydown="handleKeyDown"
        @input="adjustHeight"
        class="chat-textarea"
      ></textarea>
      <button class="btn-send" @click="handleSend" :disabled="!interviewing || streaming || !newMessage.trim()">Send</button>
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
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-2);
  align-items: center;
}

.history-select {
  max-width: 180px;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
  color: var(--text-primary);
  font-family: inherit;
  font-size: var(--type-xs);
}

.chat-readonly-bar {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
  border-top: 1px solid var(--border);
  background: var(--surface-hover);
  flex-shrink: 0;
}

.chat-readonly-bar .disabled-label {
  flex: 1;
  color: var(--text-muted);
  font-size: var(--type-xs);
}

.turn-plan-note {
  margin-top: var(--space-1);
  font-size: var(--type-xs);
  color: var(--text-muted);
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

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>