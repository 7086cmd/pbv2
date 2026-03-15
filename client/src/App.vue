<script setup lang="ts">
import { ref } from "vue";
import CategoriesView from "@/shared/components/CategoriesView.vue";
import ProblemsListView from "@/shared/components/ProblemsListView.vue";
import ProblemDetailView from "@/shared/components/ProblemDetailView.vue";

const selectedCategoryId = ref<number | null>(null);
const selectedProblemId = ref<number | null>(null);

function selectCategory(id: number) {
  selectedCategoryId.value = id;
  selectedProblemId.value = null;
}

function selectProblem(id: number) {
  selectedProblemId.value = id;
}
</script>

<template>
  <div class="app-shell">
    <header class="app-header">
      <span class="app-title">PBv2</span>
      <nav class="breadcrumb">
        <span class="crumb">Categories</span>
        <template v-if="selectedCategoryId !== null">
          <span class="crumb-sep">›</span>
          <span class="crumb crumb--active">Category #{{ selectedCategoryId }}</span>
        </template>
        <template v-if="selectedProblemId !== null">
          <span class="crumb-sep">›</span>
          <span class="crumb crumb--active">Problem #{{ selectedProblemId }}</span>
        </template>
      </nav>
    </header>

    <div class="app-body">
      <!-- Column 1: Categories -->
      <aside class="col col--categories">
        <div class="col-header">Categories</div>
        <div class="col-body">
          <CategoriesView :selected-id="selectedCategoryId" @select="selectCategory" />
        </div>
      </aside>

      <!-- Column 2: Problems list (appears when category selected) -->
      <aside class="col col--problems" :class="{ visible: selectedCategoryId !== null }">
        <div class="col-header">
          Problems
          <button
            v-if="selectedCategoryId !== null"
            class="col-close"
            @click="
              selectedCategoryId = null;
              selectedProblemId = null;
            "
            title="Close"
          >
            ✕
          </button>
        </div>
        <div class="col-body">
          <ProblemsListView
            v-if="selectedCategoryId !== null"
            :category-id="selectedCategoryId"
            :selected-id="selectedProblemId"
            @select="selectProblem"
          />
        </div>
      </aside>

      <!-- Column 3: Problem detail (appears when problem selected) -->
      <main class="col col--detail" :class="{ visible: selectedProblemId !== null }">
        <div class="col-header" v-if="selectedProblemId !== null">
          Problem #{{ selectedProblemId }}
          <button class="col-close" @click="selectedProblemId = null" title="Close">✕</button>
        </div>
        <div class="col-body">
          <ProblemDetailView v-if="selectedProblemId !== null" :problem-id="selectedProblemId" />
          <div v-else-if="selectedCategoryId !== null" class="empty-hint">
            <p>Select a problem from the list.</p>
          </div>
          <div v-else class="empty-hint empty-hint--welcome">
            <div class="welcome-icon">📚</div>
            <h2>Problem Bank</h2>
            <p>Select a category on the left to browse problems.</p>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family:
    system-ui,
    -apple-system,
    sans-serif;
  background: #f5f6fa;
  color: #1a1a1a;
}

/* ── Header ────────────────────────────────────────────────────── */

.app-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  height: 44px;
  padding: 0 1rem;
  background: #1a1a2e;
  color: #fff;
  flex-shrink: 0;
  user-select: none;
}

.app-title {
  font-size: 1rem;
  font-weight: 700;
  letter-spacing: 0.03em;
  color: #a0c4ff;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.8rem;
}

.crumb {
  color: #8899bb;
}

.crumb--active {
  color: #cdd9f5;
}

.crumb-sep {
  color: #445577;
  font-size: 0.75rem;
}

/* ── Body: three-column layout ─────────────────────────────────── */

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.col {
  display: flex;
  flex-direction: column;
  background: #fff;
  border-right: 1px solid #e0e4ee;
  overflow: hidden;
  transition:
    width 0.2s ease,
    opacity 0.2s ease;
}

.col--categories {
  width: 320px;
  flex-shrink: 0;
}

.col--problems {
  width: 0;
  opacity: 0;
  pointer-events: none;
  flex-shrink: 0;
}

.col--problems.visible {
  width: 280px;
  opacity: 1;
  pointer-events: auto;
}

.col--detail {
  flex: 1;
  background: #fafbff;
  border-right: none;
  min-width: 0;
}

/* ── Column header ─────────────────────────────────────────────── */

.col-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.55rem 0.75rem;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #888;
  background: #f8f9fd;
  border-bottom: 1px solid #e8eaf0;
  flex-shrink: 0;
}

.col-close {
  background: none;
  border: none;
  cursor: pointer;
  color: #bbb;
  font-size: 0.8rem;
  padding: 0.1rem 0.3rem;
  border-radius: 3px;
  transition:
    color 0.1s,
    background 0.1s;
  line-height: 1;
}

.col-close:hover {
  color: #555;
  background: #eee;
}

/* ── Column body ───────────────────────────────────────────────── */

.col-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* ── Empty states ──────────────────────────────────────────────── */

.empty-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  color: #aaa;
  font-size: 0.9rem;
  padding: 2rem;
  gap: 0.5rem;
}

.empty-hint--welcome {
  color: #999;
}

.welcome-icon {
  font-size: 2.5rem;
  margin-bottom: 0.5rem;
}

.empty-hint h2 {
  font-size: 1.1rem;
  font-weight: 600;
  color: #777;
  margin: 0;
}

.empty-hint p {
  margin: 0;
  font-size: 0.85rem;
}
</style>
