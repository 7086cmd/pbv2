<script setup lang="ts">
import { ref } from "vue";
import CategoriesView from "@/shared/components/CategoriesView.vue";
import ProblemsListView from "@/shared/components/ProblemsListView.vue";
import ProblemDetailView from "@/shared/components/ProblemDetailView.vue";
import ProblemEditor from "@/shared/components/ProblemEditor.vue";

const selectedCategoryId = ref<number | null>(null);
const selectedProblemId = ref<number | null>(null);
const editMode = ref(false);

function selectCategory(id: number) {
  selectedCategoryId.value = id;
  selectedProblemId.value = null;
  editMode.value = false;
}

function selectProblem(id: number) {
  selectedProblemId.value = id;
  editMode.value = false;
}

function enterEditMode() {
  editMode.value = true;
}

function onSaved() {
  editMode.value = false;
}

function onCancelled() {
  editMode.value = false;
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

      <!-- Column 3: Problem detail / editor -->
      <main class="col col--detail" :class="{ visible: selectedProblemId !== null }">
        <div class="col-header" v-if="selectedProblemId !== null">
          <span>
            Problem #{{ selectedProblemId }}
            <span v-if="editMode" class="edit-badge">editing</span>
          </span>
          <div class="col-header-actions">
            <button
              v-if="!editMode"
              class="col-action-btn"
              title="Edit problem"
              @click="enterEditMode"
            >
              ✏ Edit
            </button>
            <button
              class="col-close"
              @click="
                selectedProblemId = null;
                editMode = false;
              "
              title="Close"
            >
              ✕
            </button>
          </div>
        </div>
        <div class="col-body">
          <template v-if="selectedProblemId !== null">
            <ProblemEditor
              v-if="editMode"
              :problem-id="selectedProblemId"
              @saved="onSaved"
              @cancelled="onCancelled"
            />
            <ProblemDetailView v-else :problem-id="selectedProblemId" />
          </template>
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

.col-header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.edit-badge {
  display: inline-block;
  font-size: 0.65rem;
  font-weight: 600;
  color: #2563eb;
  background: #dbeafe;
  border-radius: 3px;
  padding: 1px 5px;
  margin-left: 6px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  vertical-align: middle;
}

.col-action-btn {
  background: none;
  border: 1px solid #c8ccda;
  cursor: pointer;
  color: #555;
  font-size: 0.75rem;
  padding: 0.15rem 0.55rem;
  border-radius: 4px;
  transition:
    color 0.1s,
    background 0.1s,
    border-color 0.1s;
  line-height: 1.4;
}

.col-action-btn:hover {
  color: #1a1a1a;
  background: #e8eaf0;
  border-color: #aaa;
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
