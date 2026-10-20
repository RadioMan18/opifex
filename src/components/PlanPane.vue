<script setup lang="ts">
import { computed, ref } from 'vue';
import { PropType } from 'vue';
import type { AgentPlan, PlanVersion } from '@/types/agent';

const props = defineProps({
  plan: { type: Object as PropType<AgentPlan | null>, required: true, default: null },
  agentName: { type: String, required: true },
  versions: { type: Array as PropType<any[]>, required: true, default: () => [] },
  compareVersionId: { type: String as PropType<string | null>, required: false, default: null },
  compareVersion: { type: Object as PropType<PlanVersion | null>, required: false, default: null },
});

const emit = defineEmits<{
  update: [section: keyof AgentPlan, value: string | string[]],
  'save-version': [],
  'restore-version': [versionId: string],
  'compare-version': [versionId: string | null],
  'export-agent': [],
}>();

const plan = computed(() => props.plan);
const compareVersion = computed(() => props.compareVersion);
const showVersions = ref(false);

const planGoal = computed({
  get: () => plan.value?.goal || '',
  set: (val: string) => emit('update', 'goal', val),
});

const planSteps = computed({
  get: () => formatArray(plan.value?.steps),
  set: (val: string) => emit('update', 'steps', val.split(',').map(s => s.trim()).filter(Boolean)),
});

const planTools = computed({
  get: () => formatArray(plan.value?.tools),
  set: (val: string) => emit('update', 'tools', val.split(',').map(s => s.trim()).filter(Boolean)),
});

const planKnowledge = computed({
  get: () => formatArray(plan.value?.knowledgeSources),
  set: (val: string) => emit('update', 'knowledgeSources', val.split(',').map(s => s.trim()).filter(Boolean)),
});

const planTriggers = computed({
  get: () => formatArray(plan.value?.triggers),
  set: (val: string) => emit('update', 'triggers', val.split(',').map(s => s.trim()).filter(Boolean)),
});

const planOutputs = computed({
  get: () => formatArray(plan.value?.outputs),
  set: (val: string) => emit('update', 'outputs', val.split(',').map(s => s.trim()).filter(Boolean)),
});

function formatArray(val: string[] | undefined): string {
  return (val || []).join(', ');
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}
</script>

<template>
  <div class="plan-pane" v-if="plan">
    <div class="plan-header">
      <div class="plan-title">
        <span class="plan-icon">📋</span>
        <h2>{{ props.agentName }} Plan</h2>
      </div>
      <div class="plan-actions">
        <button class="btn-version" @click="showVersions = !showVersions" :class="{ active: showVersions }">
          📚 Versions
        </button>
        <button class="btn-save-version" @click="emit('save-version')" title="Save version to history & file">💾 Save Version</button>
        <button class="btn-export-agent" @click="emit('export-agent')" title="Save agent to JSON file">💾 Save Agent</button>
      </div>
    </div>

    <div class="plan-content">
      <div class="plan-section">
        <label class="section-label">Goal</label>
        <textarea
          v-model="planGoal"
          class="plan-input"
          placeholder="What does this agent accomplish?"
          rows="3"
        ></textarea>
      </div>

      <div class="plan-section">
        <label class="section-label">Steps</label>
        <textarea
          v-model="planSteps"
          class="plan-input"
          placeholder="Step 1, Step 2, Step 3"
          rows="4"
        ></textarea>
      </div>

      <div class="plan-section">
        <label class="section-label">Tools</label>
        <textarea
          v-model="planTools"
          class="plan-input"
          placeholder="knowledge_base, ticket_system, crm"
          rows="3"
        ></textarea>
      </div>

      <div class="plan-section">
        <label class="section-label">Knowledge Sources</label>
        <textarea
          v-model="planKnowledge"
          class="plan-input"
          placeholder="faq.md, product_docs.pdf, pricing.md"
          rows="3"
        ></textarea>
      </div>

      <div class="plan-section">
        <label class="section-label">Triggers</label>
        <textarea
          v-model="planTriggers"
          class="plan-input"
          placeholder="customer_message, new_lead, meeting_request"
          rows="3"
        ></textarea>
      </div>

      <div class="plan-section">
        <label class="section-label">Outputs</label>
        <textarea
          v-model="planOutputs"
          class="plan-input"
          placeholder="response, ticket, report, chart"
          rows="3"
        ></textarea>
      </div>
    </div>

    <div v-if="showVersions" class="version-history">
      <h4>Plan Versions</h4>
      <div class="version-list">
        <div
          v-for="v of props.versions"
          :key="v.id"
          class="version-item"
        >
          <div class="version-info">
            <div class="version-label">{{ v.label || 'Untitled version' }}</div>
            <div class="version-date">{{ formatDate(v.created_at) }}</div>
          </div>
          <button class="btn-restore" @click="emit('restore-version', v.id)">Restore</button>
          <button class="btn-compare" @click="emit('compare-version', v.id)">Compare</button>
        </div>
      </div>
      <div v-if="props.versions.length === 0" class="version-empty">
        No saved versions yet.
      </div>
    </div>

    <div v-if="compareVersion" class="plan-diff-view">
      <h4>Plan Version Comparison</h4>
      <div class="diff-info">
        <span class="diff-current">Current: {{ plan?.goal || 'No plan' }}</span>
        <span class="diff-arrow">↔</span>
        <span class="diff-version">{{ compareVersion.label || 'Untitled version' }}: {{ compareVersion.specJson?.goal || 'No plan' }}</span>
      </div>
      <div class="diff-sections">
        <div class="diff-section">
          <h5>Goal</h5>
          <div class="diff-current-text">{{ plan?.goal || '—' }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ compareVersion?.specJson?.goal || '—' }}</div>
        </div>
        <div class="diff-section">
          <h5>Steps</h5>
          <div class="diff-current-text">{{ (plan?.steps || []).join(', ') || '—' }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ (compareVersion?.specJson?.steps || []).join(', ') || '—' }}</div>
        </div>
        <div class="diff-section">
          <h5>Tools</h5>
          <div class="diff-current-text">{{ (plan?.tools || []).join(', ') || '—' }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ (compareVersion?.specJson?.tools || []).join(', ') || '—' }}</div>
        </div>
        <div class="diff-section">
          <h5>Knowledge Sources</h5>
          <div class="diff-current-text">{{ (plan?.knowledgeSources || []).join(', ') || '—' }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ (compareVersion?.specJson?.knowledgeSources || []).join(', ') || '—' }}</div>
        </div>
        <div class="diff-section">
          <h5>Triggers</h5>
          <div class="diff-current-text">{{ (plan?.triggers || []).join(', ') || '—' }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ (compareVersion?.specJson?.triggers || []).join(', ') || '—' }}</div>
        </div>
        <div class="diff-section">
          <h5>Outputs</h5>
          <div class="diff-current-text">{{ (plan?.outputs || []).join(', ') || '—' }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ (compareVersion?.specJson?.outputs || []).join(', ') || '—' }}</div>
        </div>
      </div>
      <button class="btn-close-diff" @click="emit('compare-version', null)">Close Comparison</button>
    </div>
  </div>

  <div v-else class="plan-empty">
    <div class="empty-icon">💬</div>
    <p>Start a conversation with Oppy to build your agent plan.</p>
  </div>
</template>

<style scoped>
.plan-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-secondary);
}

.plan-header {
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-shrink: 0;
  background: var(--bg-tertiary);
}

.plan-icon {
  font-size: 1.5rem;
}

.plan-title h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.plan-actions {
  margin-left: auto;
  display: flex;
  gap: 0.5rem;
}

.plan-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.plan-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.section-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.plan-input {
  width: 100%;
  padding: 0.75rem 1rem;
  border: 1px solid var(--input-border);
  border-radius: 8px;
  font-size: 0.95rem;
  font-family: inherit;
  resize: vertical;
  min-height: 80px;
  transition: border-color 0.2s, box-shadow 0.2s;
  background: var(--input-bg);
  color: var(--text-primary);
}

.plan-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.version-history {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border);
  background: var(--bg-tertiary);
}

.version-history h4 {
  margin: 0 0 0.75rem 0;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-height: 200px;
  overflow-y: auto;
}

.version-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.625rem 0.875rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-secondary);
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.version-label {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-primary);
}

.version-date {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.btn-restore {
  padding: 0.375rem 0.75rem;
  font-size: 0.8rem;
  background: var(--accent);
  color: #ffffff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.btn-restore:hover {
  background: var(--accent-hover);
}

.btn-compare {
  padding: 0.375rem 0.75rem;
  font-size: 0.8rem;
  background: var(--bg-muted);
  color: var(--text-secondary);
  border: 1px solid var(--input-border);
  border-radius: 4px;
  cursor: pointer;
}

.btn-compare:hover {
  background: var(--border-hover);
}

.version-empty {
  font-size: 0.85rem;
  color: var(--text-muted);
  text-align: center;
  padding: 1rem;
}

.btn-version, .btn-save-version, .btn-export-agent {
  padding: 0.5rem 0.875rem;
  font-size: 0.8rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.btn-version {
  background: var(--bg-muted);
  color: var(--text-secondary);
}
.btn-version:hover {
  background: var(--border-hover);
}
.btn-version.active {
  background: var(--bg-muted);
  color: var(--accent);
}

.btn-save-version {
  background: var(--success);
  color: #ffffff;
}
.btn-save-version:hover {
  background: var(--success-hover);
}

.btn-export-agent {
  background: var(--accent);
  color: #ffffff;
}
.btn-export-agent:hover {
  background: var(--accent-hover);
}

.plan-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.plan-empty p {
  font-size: 0.95rem;
}

.plan-diff-view {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border);
  background: var(--bg-tertiary);
}

.diff-info {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  margin-bottom: 1rem;
  padding: 0.75rem;
  background: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border);
}

.diff-current,
.diff-version {
  font-size: 0.85rem;
  color: var(--text-secondary);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.diff-current {
  font-weight: 600;
  color: var(--text-primary);
}

.diff-version {
  font-weight: 600;
  color: var(--accent);
}

.diff-arrow {
  font-size: 0.8rem;
  color: var(--text-muted);
}

.diff-sections {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.diff-section {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  padding: 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.diff-section h5 {
  margin: 0;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-secondary);
  min-width: 140px;
}

.diff-current-text,
.diff-version-text {
  font-size: 0.85rem;
  color: var(--text-primary);
  word-break: break-all;
}

.diff-current-text {
  font-weight: 500;
}

.diff-version-text {
  font-weight: 500;
}

.btn-close-diff {
  margin-top: 1rem;
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  background: var(--danger);
  color: #ffffff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn-close-diff:hover {
  background: var(--danger-hover);
}
</style>
