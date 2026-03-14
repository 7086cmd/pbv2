<script setup lang="ts">
import { ref, onMounted } from "vue";
import { listCategories, type CategoryItem } from "../api";

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
          <th>ID</th>
          <th>Curriculum</th>
          <th>Subject</th>
          <th>Type</th>
          <th>Grade</th>
          <th>Tags</th>
          <th class="num">Problems</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in categories" :key="row.id">
          <td class="num muted">{{ row.id }}</td>
          <td>{{ row.curriculum_name || "—" }}</td>
          <td>{{ row.subject_name || "—" }}</td>
          <td>
            <span class="badge" :class="'badge--' + row.subject_category">
              {{ CATEGORY_LABELS[row.subject_category] ?? row.subject_category }}
            </span>
          </td>
          <td class="num">{{ row.grade }}</td>
          <td>
            <span v-for="tag in row.categories" :key="tag" class="tag">{{ tag }}</span>
            <span v-if="row.categories.length === 0" class="muted">—</span>
          </td>
          <td class="num">{{ row.problem_count }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.categories-wrap {
  overflow-x: auto;
}

.pb-state {
  color: #888;
  font-size: 0.9rem;
  padding: 0.5rem 0;
}

.pb-error {
  color: #c0392b;
  font-size: 0.9rem;
  padding: 0.5rem 0;
}

.cat-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.cat-table th,
.cat-table td {
  text-align: left;
  padding: 0.45rem 0.75rem;
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

.cat-table tbody tr:hover {
  background: #f5f7ff;
}

.num {
  text-align: right;
}

.muted {
  color: #aaa;
}

.badge {
  display: inline-block;
  padding: 0.15rem 0.55rem;
  border-radius: 9999px;
  font-size: 0.75rem;
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

.tag {
  display: inline-block;
  margin: 0.1rem 0.2rem 0.1rem 0;
  padding: 0.1rem 0.45rem;
  border-radius: 4px;
  font-size: 0.75rem;
  background: #eee;
  color: #444;
}
</style>
