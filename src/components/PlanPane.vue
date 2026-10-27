<script setup lang="ts">
import { computed, ref } from 'vue';
import { PropType } from 'vue';
import type { AgentPlan, AgentPlanStep, PlanVersion } from '@/types/agent';

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
        <svg class="plan-icon" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="12 2 2 7 12 12 22 7 12 2"/>
          <polyline points="2 17 12 22 22 17"/>
          <polyline points="2 12 12 17 22 12"/>
        </svg>
        <h2>{{ props.agentName }} Plan</h2>
      </div>
      <div class="plan-actions">
        <button class="btn-version" @click="showVersions = !showVersions" :class="{ active: showVersions }">
          Versions
        </button>
        <button class="btn-save-version" @click="emit('save-version')" title="Save version to history & file">Save Version</button>
        <button class="btn-export-agent" @click="emit('export-agent')" title="Save agent to JSON file">Save Agent</button>
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
        <ol v-if="plan?.steps && plan.steps.length" class="plan-steps-preview">
          <li v-for="(step, idx) in plan.steps" :key="idx">{{ step.action }}</li>
        </ol>
        <p v-else class="plan-steps-empty">—</p>
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
          <div class="diff-current-text">{{ (plan?.steps || []).map((s: AgentPlanStep) => s.action).join(', ') || '—' }}</div>
          <div class="diff-arrow">&harr;</div>
          <div class="diff-version-text">{{ (compareVersion?.specJson?.steps || []).map((s: AgentPlanStep) => s.action).join(', ') || '—' }}</div>
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
    <div class="empty-icon">
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="12 2 2 7 12 12 22 7 12 2"/>
        <polyline points="2 17 12 22 22 17"/>
        <polyline points="2 12 12 17 22 12"/>
      </svg>
    </div>
    <p>Start a conversation with Oppy to build your agent plan.</p>
  </div>
</template>

<style scoped>
.plan-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--surface-card);
}

.plan-header {
  padding: var(--space-5) var(--space-6);
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;
  background: var(--surface-hover);
}

.plan-icon {
  width: 24px;
  height: 24px;
  color: var(--primary);
}

.plan-title h2 {
  margin: 0;
  font-size: var(--type-xl);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.plan-actions {
  margin-left: auto;
  display: flex;
  gap: var(--space-2);
}

.plan-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-6);
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
}

.plan-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.section-label {
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.plan-input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  font-size: var(--type-base);
  font-family: inherit;
  resize: vertical;
  min-height: 80px;
  transition: border-color 0.2s, box-shadow 0.2s;
  background: var(--surface-card);
  color: var(--text-primary);
}

.plan-input:focus {
  outline: none;
  border-color: var(--border-focus);
  box-shadow: 0 0 0 3px rgba(123, 179, 232, 0.1);
}

.plan-steps-preview {
  margin: 0;
  padding-left: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.plan-steps-empty {
  color: var(--text-muted);
  font-style: italic;
  margin: 0;
}

.version-history {
  padding: var(--space-4) var(--space-6);
  border-top: 1px solid var(--border);
  background: var(--surface-hover);
}

.version-history h4 {
  margin: 0 0 var(--space-3) 0;
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  max-height: 200px;
  overflow-y: auto;
}

.version-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  background: var(--surface-card);
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.version-label {
  font-size: var(--type-base);
  font-weight: 500;
  color: var(--text-primary);
}

.version-date {
  font-size: var(--type-xs);
  color: var(--text-secondary);
}

.btn-restore {
  padding: var(--space-1) var(--space-3);
  font-size: var(--type-sm);
  background: var(--primary);
  color: var(--primary-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 600;
}

.btn-restore:hover {
  background: var(--primary-hover);
}

.btn-compare {
  padding: var(--space-1) var(--space-3);
  font-size: var(--type-sm);
  background: var(--surface-card);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-weight: 500;
}

.btn-compare:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.version-empty {
  font-size: var(--type-sm);
  color: var(--text-muted);
  text-align: center;
  padding: var(--space-4);
}

.btn-version, .btn-save-version, .btn-export-agent {
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  transition: background 0.2s, border-color 0.2s;
}

.btn-version {
  background: var(--surface-card);
  color: var(--text-secondary);
}

.btn-version:hover {
  background: var(--surface-hover);
}

.btn-version.active {
  background: var(--surface-hover);
  color: var(--primary);
}

.btn-save-version {
  background: var(--success);
  color: var(--success-fg);
}

.btn-save-version:hover {
  background: #7BB3E8;
}

.btn-export-agent {
  background: var(--primary);
  color: var(--primary-fg);
}

.btn-export-agent:hover {
  background: var(--primary-hover);
}

.plan-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  padding: var(--space-8);
}

.empty-icon {
  width: 48px;
  height: 48px;
  color: var(--text-muted);
  margin-bottom: var(--space-3);
}

.plan-empty p {
  font-size: var(--type-base);
  margin: 0;
}

.plan-diff-view {
  padding: var(--space-4) var(--space-6);
  border-top: 1px solid var(--border);
  background: var(--surface-hover);
}

.diff-info {
  display: flex;
  gap: var(--space-3);
  align-items: center;
  margin-bottom: var(--space-4);
  padding: var(--space-3);
  background: var(--surface-card);
  border-radius: var(--radius-md);
  border: 1px solid var(--border);
}

.diff-current,
.diff-version {
  font-size: var(--type-sm);
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
  color: var(--primary);
}

.diff-arrow {
  font-size: var(--type-xs);
  color: var(--text-muted);
}

.diff-sections {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.diff-section {
  display: flex;
  gap: var(--space-3);
  align-items: center;
  padding: var(--space-3);
  background: var(--surface-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}

.diff-section h5 {
  margin: 0;
  font-size: var(--type-xs);
  font-weight: 600;
  color: var(--text-secondary);
  min-width: 140px;
}

.diff-current-text,
.diff-version-text {
  font-size: var(--type-sm);
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
  margin-top: var(--space-4);
  padding: var(--space-2) var(--space-4);
  font-size: var(--type-sm);
  background: var(--danger);
  color: var(--danger-fg);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-weight: 600;
}

.btn-close-diff:hover {
  background: var(--danger);
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>