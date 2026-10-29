<template>
  <div class="p-4 bg-slate-900 border-t border-slate-800 text-slate-200 h-full flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-sm font-semibold">Skills</h3>
      <button
        @click="showAddForm = true"
        class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 rounded text-white"
      >
        + Add Skill
      </button>
    </div>

    <div class="flex-1 overflow-y-auto space-y-2">
      <div v-if="skills.length === 0" class="flex items-center justify-center py-10">
        <p class="text-slate-500 text-sm">No skills defined yet.</p>
      </div>

      <div v-for="skill in skills" :key="skill.id" class="mb-3">
        <div
          class="p-3 rounded bg-slate-800 border border-slate-700 hover:border-slate-600 transition-colors"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-purple-500"></span>
              <span class="text-sm font-medium text-slate-200">{{ skill.name }}</span>
              <span
                class="text-[10px] text-slate-500 uppercase tracking-tighter"
                >{{ skill.kind }}</span>
            </div>
            <button
              @click="removeSkill(skill.id)"
              class="text-[10px] text-red-400 hover:text-red-300 hover:underline"
            >
              Delete
            </button>
          </div>

          <div v-if="skill.description" class="mb-2">
            <span class="text-[10px] text-slate-400 text-truncate"
              :title="skill.description">
              {{ skill.description }}
            </span>
          </div>

          <div class="flex items-center gap-3 mb-2">
            <span class="text-[10px] text-slate-400">AI Editing:</span>
            <span
              :class="skill.ai_editing_enabled ? 'text-green-400' : 'text-slate-500'"
            >
              {{ skill.ai_editing_enabled ? 'Enabled' : 'Disabled' }}
            </span>
          </div>

          <div class="mt-2">
            <div class="text-[10px] text-slate-400 mb-1">Instructions:</span>
            <div class="text-xs text-slate-300 bg-slate-950 border border-slate-700 rounded px-2 py-1.5 break-all">
              {{ skill.instructions }}
            </div>
          </div>

          <div v-if="skill.attached_to.length > 0" class="mt-2">
            <div class="text-[10px] text-slate-400 mb-1">Attached to:</span>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="aid in skill.attached_to"
                :key="aid"
                class="px-1.5 py-0.5 rounded bg-slate-800/50 border border-slate-700/50 text-[9px] text-slate-300"
              >
                {{ aid }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Skill Form -->
    <div v-if="showAddForm" class="mt-4 p-4 rounded bg-slate-800/50 border border-slate-700/50">
      <h4 class="text-xs font-bold text-slate-400 uppercase tracking-wider mb-3">
        Add Skill
      </h4>
      <div class="space-y-3">
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Name</label>
          <input
            v-model="newSkill.name"
            type="text"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="e.g. Summarize Web Pages"
          />
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Description</label>
          <input
            v-model="newSkill.description"
            type="text"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="e.g. Summarizes web content into bullet points"
          />
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Kind</label>
          <select
            v-model="newSkill.kind"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
          >
            <option value="create-with-ai">Create With AI</option>
            <option value="upload-files">Upload Files</option>
            <option value="write-instructions">Write Instructions</option>
            <option value="add-existing">Add Existing</option>
          </select>
        </div>
        <div>
          <label class="text-[10px] text-slate-300 block mb-1">Instructions</label>
          <textarea
            v-model="newSkill.instructions"
            rows="4"
            class="w-full bg-slate-950 border border-slate-600 rounded px-2 py-1.5 text-xs text-slate-200 focus:border-indigo-500 focus:outline-none"
            placeholder="Describe what this skill does..."
          ></textarea>
        </div>
        <div class="flex items-center gap-2 mb-2">
          <input
            v-model="newSkill.ai_editing_enabled"
            type="checkbox"
            class="rounded bg-slate-900 border-slate-600 text-indigo-600 focus:ring-indigo-500"
          />
          <label class="text-xs text-slate-300 cursor-pointer">
            AI Skill Editing (agent can create/edit skills)
          </label>
        </div>
        <div class="flex justify-end gap-2 mt-4">
          <button
            @click="showAddForm = false"
            class="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded text-slate-200"
          >
            Cancel
          </button>
          <button
            @click="saveSkill"
            class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 rounded text-white"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { createSkill, listSkills } from '@/services/ipc';
import type { Skill, SkillKind } from '@/types/agent';

const skills = ref<Skill[]>([]);
const showAddForm = ref(false);
const newSkill = ref({
  name: '',
  description: '',
  kind: 'create-with-ai' as SkillKind,
  ai_editing_enabled: false,
  instructions: '',
});

async function loadSkills() {
  if (!props.agentId) return;
  try {
    skills.value = (await listSkills({ agent_id: props.agentId })) as Skill[];
  } catch (e) {
    console.error('Failed to load skills:', e);
  }
}

async function saveSkill() {
  if (!props.agentId || !newSkill.value.name.trim()) return;

  try {
    const created = await createSkill({
      agent_id: props.agentId,
      name: newSkill.value.name,
      description: newSkill.value.description || undefined,
      kind: newSkill.value.kind,
      ai_editing_enabled: newSkill.value.ai_editing_enabled,
      instructions: newSkill.value.instructions,
    });
    skills.value.push(created as Skill);
    showAddForm.value = false;
    newSkill.value = {
      name: '',
      description: '',
      kind: 'create-with-ai' as SkillKind,
      ai_editing_enabled: false,
      instructions: '',
    };
  } catch (e: any) {
    console.error('Failed to create skill:', e);
  }
}

async function removeSkill(id: string) {
  if (!confirm('Delete this skill?')) return;
  try {
    // deleteSkill not yet exposed in IPC - would need to add
    console.log('Delete skill:', id);
  } catch (e: any) {
    console.error('Failed to delete skill:', e);
  }
}

const props = defineProps<{
  agentId: string | undefined;
}>();

loadSkills();
</script>

<style scoped>
</style>
