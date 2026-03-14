<script setup lang="ts">
import { CategoriesView, ProblemGroupView, QuestionView } from "@/shared";
</script>

<template>
  <main>
    <header>
      <h1>PBv2</h1>
    </header>
    <section>
      <h2>Categories</h2>
      <CategoriesView />
    </section>
    <section>
      <h2>Problem Group</h2>
      <ProblemGroupView />
    </section>
    <section>
      <h2>Single Problem</h2>
      <QuestionView />
    </section>
  </main>
</template>

<style scoped>
main {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem 1rem;
  font-family: system-ui, sans-serif;
}

header {
  border-bottom: 2px solid #eee;
  margin-bottom: 2rem;
  padding-bottom: 1rem;
}

h1 {
  margin: 0;
  font-size: 1.75rem;
  font-weight: 700;
  color: #1a1a2e;
}

h2 {
  font-size: 1.2rem;
  font-weight: 600;
  color: #444;
  margin: 2rem 0 1rem;
  padding-bottom: 0.25rem;
  border-bottom: 1px solid #eee;
}

section {
  margin-bottom: 3rem;
}
</style>
