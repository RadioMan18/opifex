<script setup lang="ts">
import { PropType } from 'vue';
import type { Message } from '@/types/agent';

defineProps({
  message: { type: Object as PropType<Message>, required: true },
  streaming: { type: Boolean, required: false, default: false },
});
</script>

<template>
  <div class="message" :class="[message.role, { streaming }]">
    <div class="message-avatar" :class="message.role">
      {{ message.role === 'user' ? 'U' : 'O' }}
    </div>
    <div class="message-content">
      <div class="message-text">{{ message.content }}</div>
      <div class="message-meta">
        <span class="message-role">{{ message.role === 'user' ? 'You' : 'Oppy' }}</span>
        <span class="message-time">{{ new Date(message.createdAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) }}</span>
        <span v-if="streaming" class="streaming-indicator">● typing</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
  animation: pulse 1.5s infinite;
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
