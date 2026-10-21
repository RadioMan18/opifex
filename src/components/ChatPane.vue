<script setup lang="ts">
import { ref } from 'vue';
import { PropType } from 'vue';
import type { Message } from '@/types/agent';

defineProps({
  messages: { type: Array as PropType<Message[]>, required: true },
  streaming: { type: Boolean, required: true },
  streamingMsgId: { type: String, required: true },
  currentAgentName: { type: String, required: false, default: '' },
  interviewing: { type: Boolean, required: false, default: false },
  buildError: { type: [String] as PropType<string | null>, required: false, default: null },
});

const emit = defineEmits(['send', 'build', 'retry', 'new-conversation']);
const newMessage = ref('');
const textareaRef = ref<HTMLTextAreaElement | null>(null);

function adjustHeight() {
  if (!textareaRef.value) return;
  textareaRef.value.style.height = 'auto';
  textareaRef.value.style.height = `${Math.min(textareaRef.value.scrollHeight, 160)}px`;
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
    <div class="chat-header">
      <div class="chat-title">
        <span class="chat-icon">🤖</span>
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
        <button v-if="currentAgentName" class="btn-new-chat" @click="emit('new-conversation')" title="Start a fresh conversation and clear plan">💬 New Chat</button>
        <button v-if="interviewing" class="btn-build" @click="handleBuild">Build Agent</button>
        <button v-if="!interviewing && buildError" class="btn-retry" @click="handleRetry">Retry</button>
        <span v-if="!interviewing && buildError" class="build-error">{{ buildError }}</span>
      </div>
    </div>

    <div class="agent-library-placeholder" v-if="!currentAgentName">
      <div class="empty-state">
        <div class="empty-icon">📦</div>
        <p>Select or create an agent from the library to start building with Oppy.</p>
      </div>
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
              <span class="plan-card-icon">📋</span>
              <span class="plan-card-title">Agent Plan</span>
            </div>
            <div v-if="msg.turnPlanJson.goal" class="plan-card-goal">
              <strong>Goal:</strong> {{ msg.turnPlanJson.goal }}
            </div>
            <div v-if="msg.turnPlanJson.steps && msg.turnPlanJson.steps.length" class="plan-card-steps">
              <strong>Steps ({{ msg.turnPlanJson.steps.length }}):</strong>
              <ol class="plan-steps-preview">
                <li v-for="(step, idx) in msg.turnPlanJson.steps.slice(0, 3)" :key="idx">{{ step }}</li>
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

    <div class="chat-input-area" v-if="currentAgentName">
      <textarea
        ref="textareaRef"
        v-model="newMessage"
        placeholder="Tell Oppy what you want your agent to do... (Shift+Enter for new line)"
        rows="1"
        @keydown="handleKeyDown"
        @input="adjustHeight"
        class="chat-textarea"
      ></textarea>
      <button class="btn-send" @click="handleSend" :disabled="streaming || !newMessage.trim()">Send</button>
    </div>
  </div>
</template>

<style scoped>
.chat-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid #e5e5e5;
  overflow: hidden;
}

.chat-header {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
  background: var(--bg-tertiary);
}

.chat-icon {
  font-size: 1.5rem;
}

.chat-title h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.chat-subtitle {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin-top: 0.125rem;
}

.chat-actions {
  margin-left: auto;
}

.btn-build {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 3px solid var(--border);
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn-build:hover {
  background: #F0ECE6;
}

.btn-retry {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 3px solid var(--border);
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn-retry:hover {
  background: #F0ECE6;
}

.build-error {
  font-size: 0.8rem;
  color: var(--danger);
  margin-left: 0.75rem;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-new-chat {
  padding: 0.5rem 0.875rem;
  font-size: 0.875rem;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 3px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.375rem;
  transition: all 0.2s;
}

.btn-new-chat:hover {
  background: #F0ECE6;
  border-color: var(--border-hover);
}

.model-select {
  padding: 0.4rem 0.75rem;
  border: 1px solid var(--input-border);
  border-radius: 6px;
  font-size: 0.9rem;
  background: var(--input-bg);
  color: var(--text-primary);
  min-width: 140px;
}

.agent-library-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state {
  text-align: center;
  color: var(--text-muted);
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.empty-state p {
  font-size: 0.95rem;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.message {
  display: flex;
  gap: 0.75rem;
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
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.85rem;
  font-weight: 600;
  flex-shrink: 0;
}

.message.user .message-avatar {
  background: var(--accent);
  color: #ffffff;
}

.message.assistant .message-avatar {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.message-content {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
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
  padding: 0.625rem 0.875rem;
  border-radius: 12px;
  font-size: 0.95rem;
  line-height: 1.5;
  white-space: pre-wrap;
}

.message.user .message-text {
  background: var(--accent);
  color: #ffffff;
  border-bottom-right-radius: 4px;
}

.message.assistant .message-text {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-bottom-left-radius: 4px;
}

.message-meta {
  display: flex;
  gap: 0.5rem;
  font-size: 0.7rem;
  color: var(--text-muted);
}

.message.user .message-meta {
  justify-content: flex-end;
}

.message.assistant .message-meta {
  justify-content: flex-start;
}

.streaming-indicator {
  color: var(--accent);
  font-size: 0.7rem;
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
}

.typing-dots {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.typing-dots i {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.35;
  animation: typing-dot 1s infinite ease-in-out;
}

.typing-dots i:nth-child(2) {
  animation-delay: 0.15s;
}

.typing-dots i:nth-child(3) {
  animation-delay: 0.3s;
}

@keyframes typing-dot {
  0%, 60%, 100% { opacity: 0.35; transform: translateY(0); }
  30% { opacity: 1; transform: translateY(-2px); }
}

.chat-input-area {
  padding: 1rem;
  border-top: 1px solid var(--border);
  background: var(--bg-tertiary);
  display: flex;
  gap: 0.75rem;
  align-items: flex-end;
  flex-shrink: 0;
}

.chat-textarea {
  flex: 1;
  min-height: 44px;
  max-height: 120px;
  padding: 0.625rem 0.875rem;
  border: 1px solid var(--input-border);
  border-radius: 8px;
  font-size: 0.95rem;
  resize: vertical;
  font-family: inherit;
  background: var(--input-bg);
  color: var(--text-primary);
}

.chat-textarea:focus {
  outline: none;
  border-color: var(--accent);
}

.btn-send {
  padding: 0.625rem 1.25rem;
  font-size: 0.95rem;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 3px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-send:hover:not(:disabled) {
  background: #F0ECE6;
}

.btn-send:disabled {
  background: var(--bg-tertiary);
  cursor: not-allowed;
}

.message-plan-card {
  margin-top: 0.75rem;
  padding: 0.75rem 1rem;
  background: rgba(59, 130, 246, 0.06);
  border: 1px solid rgba(59, 130, 246, 0.2);
  border-radius: 8px;
  font-size: 0.88rem;
  color: var(--text-primary);
}

.plan-card-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
  color: var(--accent);
  margin-bottom: 0.4rem;
}

.plan-card-goal {
  margin-bottom: 0.4rem;
  line-height: 1.4;
}

.plan-card-steps strong {
  display: block;
  margin-bottom: 0.25rem;
}

.plan-steps-preview {
  margin: 0;
  padding-left: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.more-steps {
  list-style-type: none;
  font-style: italic;
  color: var(--text-secondary);
  margin-top: 0.2rem;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
