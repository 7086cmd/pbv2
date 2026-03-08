<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { renderSingleProblem } from '../api'

const html = ref('')
const error = ref('')

onMounted(async () => {
  try {
    html.value = await renderSingleProblem()
  } catch (e) {
    error.value = String(e)
  }
})
</script>

<template>
  <div v-if="error" class="pb-error">{{ error }}</div>
  <div v-else class="pb-question" v-html="html" />
</template>

<style scoped>
.pb-question :deep(.question) {
  font-family: sans-serif;
}

.pb-question :deep(.solve-space) {
  margin-top: 0.5rem;
  color: #666;
}

.pb-question :deep(.solve-space > div) {
  border: 1px dashed #bbb;
  border-radius: 4px;
  margin-top: 0.25rem;
}

.pb-question :deep(.answer-lines > div) {
  border-bottom: 1px solid #555;
  height: 36px;
  margin-bottom: 4px;
}

.pb-error {
  color: #c00;
  padding: 1rem;
  border: 1px solid #c00;
  border-radius: 4px;
}
</style>
