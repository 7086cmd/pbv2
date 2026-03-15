<script setup lang="ts">
import { ref, onMounted } from "vue";
import { listCategories, type CategoryItem } from "../api";

const props = defineProps<{
  selectedId?: number | null;
}>();

const emit = defineEmits<{
  (e: "select", id: number): void;
}>();

const categories = ref<CategoryItem[]>([]);
const error = ref("");
const loading = ref(true);

const CATEGORY_LABELS: Record<string, string> = {
  language: "Language",
  stem: "STEM",
  humanities: "Humanities",
  arts: "Arts",
  other: "Other",
};

onMounted(async () => {
  try {
    categories.value = await listCategories();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="categories-wrap">
    <div v-if="loading" class="pb-state">Loading…</div>
    <div v-else-if="error" class="pb-error">{{ error }}</div>
    <div v-else-if="categories.length === 0" class="pb-state">No categories found.</div>
    <table v-else class="cat-table">
      <thead>
        <tr>
          <th>Curriculum</th>
          <th>Subject</th>
          <th>Type</th>
          <th>Grade</th>
          <th class="num">Problems</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="row in categories"
          :key="row.id"
          :class="{ selected: row.id === selectedId }"
          @click="emit('select', row.id)"
        >
          <td>{{ row.curriculum_name || "—" }}</td>
          <td>{{ row.subject_name || "—" }}</td>
          <td>
            <span class="badge" :class="'badge--' + row.subject_category">
              {{ CATEGORY_LABELS[row.subject_category] ?? row.subject_category }}
            </span>
          </td>
          <td class="num">{{ row.grade }}</td>
          <td class="num">{{ row.problem_count }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.categories-wrap {
  overflow-x: auto;
  overflow-y: auto;
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

.cat-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.82rem;
}

.cat-table th,
.cat-table td {
  text-align: left;
  padding: 0.4rem 0.6rem;
  border-bottom: 1px solid #eee;
  white-space: nowrap;
}

.cat-table th {
  font-weight: 600;
  color: #555;
  background: #f8f8f8;
  position: sticky;
  top: 0;
}

.cat-table tbody tr {
  cursor: pointer;
  transition: background 0.1s;
}

.cat-table tbody tr:hover {
  background: #f0f4ff;
}

.cat-table tbody tr.selected {
  background: #dce7ff;
  color: #1a3e8c;
}

.cat-table tbody tr.selected td {
  font-weight: 500;
}

.num {
  text-align: right;
}

.badge {
  display: inline-block;
  padding: 0.12rem 0.45rem;
  border-radius: 9999px;
  font-size: 0.72rem;
  font-weight: 500;
  background: #e8eaf6;
  color: #3949ab;
}

.badge--stem {
  background: #e3f2fd;
  color: #1565c0;
}
.badge--language {
  background: #f3e5f5;
  color: #6a1b9a;
}
.badge--humanities {
  background: #fce4ec;
  color: #880e4f;
}
.badge--arts {
  background: #fff8e1;
  color: #f57f17;
}
.badge--other {
  background: #f1f8e9;
  color: #33691e;
}
</style>
