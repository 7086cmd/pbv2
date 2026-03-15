<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { listProblems, type ProblemListItem } from "../api";

const props = defineProps<{
  categoryId: number;
  selectedId?: number | null;
}>();

const emit = defineEmits<{
  (e: "select", id: number): void;
}>();

const problems = ref<ProblemListItem[]>([]);
const loading = ref(true);
const error = ref("");

async function fetchProblems() {
  loading.value = true;
  error.value = "";
  problems.value = [];
  try {
    problems.value = await listProblems(props.categoryId);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(fetchProblems);
watch(() => props.categoryId, fetchProblems);
</script>

<template>
  <div class="problems-list">
    <div v-if="loading" class="pb-state">Loading…</div>
    <div v-else-if="error" class="pb-error">{{ error }}</div>
    <div v-else-if="problems.length === 0" class="pb-state">No problems in this category.</div>
    <div
      v-for="p in problems"
      :key="p.id"
      class="problem-item"
      :class="{ selected: p.id === selectedId }"
      @click="emit('select', p.id)"
    >
      <span class="problem-num">#{{ p.id }}</span>
      <span class="problem-preview">{{ p.preview }}</span>
    </div>
  </div>
</template>

<style scoped>
.problems-list {
  overflow-y: auto;
  height: 100%;
}

.pb-state {
  color: #888;
  font-size: 0.85rem;
  padding: 0.75rem;
}

.pb-error {
  color: #c0392b;
  font-size: 0.85rem;
  padding: 0.75rem;
}

.problem-item {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.55rem 0.75rem;
  border-bottom: 1px solid #eee;
  cursor: pointer;
  transition: background 0.1s;
}

.problem-item:hover {
  background: #f0f4ff;
}

.problem-item.selected {
  background: #dce7ff;
}

.problem-num {
  flex-shrink: 0;
  font-size: 0.7rem;
  color: #aaa;
  font-variant-numeric: tabular-nums;
  width: 2.5rem;
  text-align: right;
}

.problem-item.selected .problem-num {
  color: #4a70d9;
}

.problem-preview {
  font-size: 0.82rem;
  color: #333;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  line-height: 1.35;
}

.problem-item.selected .problem-preview {
  color: #1a3e8c;
  font-weight: 500;
}
</style>
