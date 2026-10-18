<script setup lang="ts">
import { ref } from 'vue';
import { PropType } from 'vue';
import type { Message } from '@/types/agent';

const props = defineProps({
  messages: { type: Array as PropType<Message[]>, required: true },
  streaming: { type: Boolean, required: true },
  streamingMsgId: { type: String, required: true },
  currentAgentName: { type: String, required: false, default: '' },
  interviewing: { type: Boolean, required: false, default: false },
  onSend: { type: Function as PropType<(content: string) => void>, required: true },
});

const emit = defineEmits(['build']);
const newMessage = ref('');

async function handleSend() {
  if (!newMessage.value.trim()) return;
  props.onSend(newMessage.value);
  newMessage.value = '';
}

async function handleBuild() {
  emit('build');
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
            {{ interviewing ? 'Interview mode — Oppy is asking questions' : 'Building agent' }}
          </div>
        </div>
      </div>
      <div class="chat-actions">
        <button v-if="interviewing" class="btn-build" @click="handleBuild">Build Agent</button>
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
          <div class="message-meta">
            <span class="message-role">{{ msg.role === 'user' ? 'You' : 'Oppy' }}</span>
            <span class="message-time">{{ formatTime(msg.createdAt) }}</span>
            <span v-if="streaming && msg.id === streamingMsgId" class="streaming-indicator">● typing</span>
          </div>
        </div>
      </div>
    </div>

    <div class="chat-input-area" v-if="currentAgentName">
      <textarea
        v-model="newMessage"
        placeholder="Tell Oppy what you want your agent to do..."
        rows="1"
        @keydown.enter.shift.prevent
        @keydown.enter="handleKeyDown"
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
  background: #ffffff;
  border-right: 1px solid #e5e5e5;
  overflow: hidden;
}

.chat-header {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #e5e5e5;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
  background: #fafafa;
}

.chat-icon {
  font-size: 1.5rem;
}

.chat-title h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: #1a1a1a;
}

.chat-subtitle {
  font-size: 0.8rem;
  color: #6b7280;
  margin-top: 0.125rem;
}

.chat-actions {
  margin-left: auto;
}

.btn-build {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: #10b981;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn-build:hover {
  background: #059669;
}

.model-select {
  padding: 0.4rem 0.75rem;
  border: 1px solid #d1d1d1;
  border-radius: 6px;
  font-size: 0.9rem;
  background: #ffffff;
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
  color: #9ca3af;
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
  background: #0066cc;
  color: #ffffff;
}

.message.assistant .message-avatar {
  background: #f1f1f3;
  color: #6b7280;
  border: 1px solid #e5e5e5;
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
  background: #0066cc;
  color: #ffffff;
  border-bottom-right-radius: 4px;
}

.message.assistant .message-text {
  background: #f1f1f3;
  color: #1a1a1a;
  border: 1px solid #e5e5e5;
  border-bottom-left-radius: 4px;
}

.message-meta {
  display: flex;
  gap: 0.5rem;
  font-size: 0.7rem;
  color: #9ca3af;
}

.message.user .message-meta {
  justify-content: flex-end;
}

.message.assistant .message-meta {
  justify-content: flex-start;
}

.streaming-indicator {
  color: #0066cc;
  animation: pulse 1.5s infinite;
  font-size: 0.7rem;
}

.chat-input-area {
  padding: 1rem;
  border-top: 1px solid #e5e5e5;
  background: #fafafa;
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
  border: 1px solid #d1d1d1;
  border-radius: 8px;
  font-size: 0.95rem;
  resize: vertical;
  font-family: inherit;
}

.chat-textarea:focus {
  outline: none;
  border-color: #0066cc;
}

.btn-send {
  padding: 0.625rem 1.25rem;
  font-size: 0.95rem;
  background: #0066cc;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
}

.btn-send:hover:not(:disabled) {
  background: #0052a3;
}

.btn-send:disabled {
  background: #9ca3af;
  cursor: not-allowed;
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
