<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from "vue";
import { renderDbProblem } from "../api";
import renderMathInElement from "katex/contrib/auto-render";
import "katex/contrib/mhchem";

const props = defineProps<{
  problemId: number;
}>();

const rendered = ref<HTMLElement | null>(null);
const html = ref("");
const loading = ref(true);
const error = ref("");

function renderMath() {
  if (!rendered.value) return;
  renderMathInElement(rendered.value, {
    delimiters: [
      { left: "$$", right: "$$", display: true },
      { left: "$", right: "$", display: false },
    ],
    throwOnError: false,
  });
}

async function fetchProblem() {
  loading.value = true;
  error.value = "";
  html.value = "";
  try {
    html.value = await renderDbProblem(props.problemId);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
  await nextTick();
  renderMath();
}

onMounted(fetchProblem);
watch(() => props.problemId, fetchProblem);
</script>

<template>
  <div class="problem-detail">
    <div v-if="loading" class="pb-state">Rendering…</div>
    <div v-else-if="error" class="pb-error">{{ error }}</div>
    <div v-else ref="rendered" class="pb-rendered" v-html="html" />
  </div>
</template>

<style scoped>
.problem-detail {
  height: 100%;
  overflow-y: auto;
}

.pb-state {
  color: #888;
  font-size: 0.9rem;
  padding: 1.5rem;
}

.pb-error {
  color: #c0392b;
  font-size: 0.875rem;
  padding: 1.5rem;
  background: #fff5f5;
  border-radius: 6px;
  margin: 1rem;
}

.pb-rendered {
  padding: 1.5rem 2rem;
  font-family: "Georgia", "Times New Roman", serif;
  font-size: 1rem;
  line-height: 1.7;
  color: #1a1a1a;
}

/* ── Rendered HTML from schema renderer ─────────────────────────────── */

.pb-rendered :deep(.question) {
  margin: 1.25rem 0;
}

.pb-rendered :deep(.problem-group) {
  font-family: inherit;
}

.pb-rendered :deep(blockquote.material) {
  border-left: 4px solid #4a90d9;
  background: #f0f7ff;
  margin: 0 0 1.5rem;
  padding: 0.75rem 1.25rem;
  border-radius: 0 6px 6px 0;
  font-style: italic;
}

.pb-rendered :deep(.answer-lines > div) {
  border-bottom: 1px solid #666;
  height: 36px;
  margin-bottom: 4px;
}

.pb-rendered :deep(.solve-space) {
  margin-top: 0.5rem;
  color: #666;
  font-size: 0.9rem;
}

.pb-rendered :deep(.proof-space) {
  border: 1px dashed #bbb;
  border-radius: 4px;
}

.pb-rendered :deep(ol) {
  padding-left: 1.75rem;
}

.pb-rendered :deep(p) {
  margin: 0.5rem 0;
}

.pb-rendered :deep(.list-label) {
  display: inline-block;
  min-width: 2rem;
  font-weight: 500;
}

.pb-rendered :deep(strong) {
  font-weight: 700;
}

.pb-rendered :deep(em) {
  font-style: italic;
}
</style>
