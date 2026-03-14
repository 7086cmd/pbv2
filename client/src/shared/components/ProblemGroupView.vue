<script setup lang="ts">
import { ref, onMounted } from "vue";
import { renderProblemGroup } from "../api";

const html = ref("");
const error = ref("");

onMounted(async () => {
  try {
    html.value = await renderProblemGroup();
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div v-if="error" class="pb-error">{{ error }}</div>
  <div v-else class="pb-problem-group" v-html="html" />
</template>

<style scoped>
.pb-problem-group :deep(.problem-group) {
  font-family: sans-serif;
}

.pb-problem-group :deep(blockquote.material) {
  border-left: 4px solid #4a90d9;
  background: #f0f7ff;
  margin: 0 0 1.5rem;
  padding: 0.75rem 1.25rem;
  border-radius: 0 4px 4px 0;
}

.pb-problem-group :deep(.question) {
  margin: 1.25rem 0;
  padding: 0.5rem 0;
}

.pb-problem-group :deep(.question-series) {
  margin: 1rem 0;
}

.pb-problem-group :deep(.answer-lines > div) {
  border-bottom: 1px solid #555;
  height: 36px;
  margin-bottom: 4px;
}

.pb-problem-group :deep(.solve-space) {
  margin-top: 0.5rem;
  color: #666;
}

.pb-problem-group :deep(.proof-space) {
  border: 1px dashed #bbb;
  border-radius: 4px;
}

.pb-error {
  color: #c00;
  padding: 1rem;
  border: 1px solid #c00;
  border-radius: 4px;
}
</style>
