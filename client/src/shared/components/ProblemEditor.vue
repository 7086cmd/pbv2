<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
import { useEditor, EditorContent, type JSONContent } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import Image from "@tiptap/extension-image";
import Underline from "@tiptap/extension-underline";
import Superscript from "@tiptap/extension-superscript";
import Subscript from "@tiptap/extension-subscript";
import { TextStyle, Color } from "@tiptap/extension-text-style";
import {
  getProblemContent,
  saveProblemContent,
  type ContentSegment,
  type ChoicePoolDto,
  type TextSegment,
  type ListSegment,
} from "../api";

// ── Props / emits ─────────────────────────────────────────────────────────────

const props = defineProps<{
  problemId: number;
}>();

const emit = defineEmits<{
  saved: [];
  cancelled: [];
}>();

// ── State ─────────────────────────────────────────────────────────────────────

const loading = ref(true);
const saving = ref(false);
const error = ref("");
const questionId = ref("");
const imageInput = ref<HTMLInputElement | null>(null);

// ── Options / choice pool ──────────────────────────────────────────────────────

const choices = ref<string[]>([]);
const orderType = ref<ChoicePoolDto["order_type"]>("uppercase_alphabetic");
const orderFormat = ref<ChoicePoolDto["order_format"]>("parenthesis");

function addChoice() {
  choices.value.push("");
}

function removeChoice(i: number) {
  choices.value.splice(i, 1);
}

function moveChoice(i: number, dir: -1 | 1) {
  const j = i + dir;
  if (j < 0 || j >= choices.value.length) return;
  [choices.value[i], choices.value[j]] = [choices.value[j]!, choices.value[i]!];
}

// ── Content segment ↔ Tiptap JSON conversion ──────────────────────────────────

type Mark = { type: string; attrs?: Record<string, unknown> };

function textSegmentToMark(seg: TextSegment): Mark[] {
  const marks: Mark[] = [];
  if (seg.bold) marks.push({ type: "bold" });
  // italic and formula both need the italic mark; formula also needs code
  if (seg.italic || seg.formula) marks.push({ type: "italic" });
  if (seg.underline) marks.push({ type: "underline" });
  if (seg.strikethrough) marks.push({ type: "strike" });
  if (seg.superscript) marks.push({ type: "superscript" });
  if (seg.subscript) marks.push({ type: "subscript" });
  // monospace and formula both need the code mark
  if (seg.monospace || seg.formula) marks.push({ type: "code" });
  if (seg.red) marks.push({ type: "textStyle", attrs: { color: "#e53e3e" } });
  return marks;
}

function segmentToInlineNode(seg: ContentSegment): JSONContent | null {
  if (seg.kind === "text") {
    const marks = textSegmentToMark(seg);
    return { type: "text", text: seg.text, marks: marks.length ? marks : undefined };
  } else if (seg.kind === "image") {
    const src = seg.data_uri ?? seg.url ?? "";
    const attrs: Record<string, unknown> = { src, alt: "" };
    if (seg.width_ratio != null) attrs.style = `width: ${Math.round(seg.width_ratio * 100)}%;`;
    return { type: "image", attrs };
  }
  return null;
}

function segmentsToDoc(segments: ContentSegment[]): JSONContent {
  const blocks: JSONContent[] = [];
  let inlineBuffer: JSONContent[] = [];

  const flushParagraph = () => {
    if (inlineBuffer.length > 0) {
      blocks.push({ type: "paragraph", content: inlineBuffer });
      inlineBuffer = [];
    }
  };

  for (const seg of segments) {
    if (seg.kind === "text" || seg.kind === "image") {
      const node = segmentToInlineNode(seg);
      if (node) inlineBuffer.push(node);
    } else if (seg.kind === "list") {
      flushParagraph();
      const listType = seg.order_type === "unordered" ? "bulletList" : "orderedList";
      blocks.push({
        type: listType,
        content: seg.items.map((itemSegs) => ({
          type: "listItem",
          content: [
            {
              type: "paragraph",
              content: itemSegs.flatMap((s) => {
                const node = segmentToInlineNode(s);
                return node ? [node] : [];
              }),
            },
          ],
        })),
      });
    }
  }

  flushParagraph();
  if (blocks.length === 0) blocks.push({ type: "paragraph" });
  return { type: "doc", content: blocks };
}

function inlineNodesToSegments(nodes: JSONContent[]): ContentSegment[] {
  const segs: ContentSegment[] = [];
  for (const node of nodes) {
    if (node.type === "text") {
      const markTypes = (node.marks ?? []).map((m: { type: string }) => m.type);
      const colorMark = (node.marks ?? []).find(
        (m: { type: string; attrs?: Record<string, unknown> }) => m.type === "textStyle",
      ) as { type: string; attrs?: { color?: string } } | undefined;
      const hasItalic = markTypes.includes("italic");
      const hasCode = markTypes.includes("code");
      segs.push({
        kind: "text",
        text: node.text ?? "",
        bold: markTypes.includes("bold"),
        italic: hasItalic && !hasCode,
        underline: markTypes.includes("underline"),
        underwave: false,
        strikethrough: markTypes.includes("strike"),
        superscript: markTypes.includes("superscript"),
        subscript: markTypes.includes("subscript"),
        monospace: hasCode && !hasItalic,
        formula: hasItalic && hasCode,
        red: colorMark?.attrs?.color === "#e53e3e",
      });
    } else if (node.type === "image") {
      const src = (node.attrs?.src as string | undefined) ?? "";
      const style = (node.attrs?.style as string | undefined) ?? "";
      const widthMatch = style.match(/width:\s*([\d.]+)%/);
      const width_ratio = widthMatch?.[1] ? parseFloat(widthMatch[1]) / 100 : null;
      segs.push(
        src.startsWith("data:")
          ? { kind: "image", url: null, data_uri: src, width_ratio }
          : { kind: "image", url: src, data_uri: null, width_ratio },
      );
    }
  }
  return segs;
}

function docToSegments(doc: JSONContent): ContentSegment[] {
  const segments: ContentSegment[] = [];

  for (const block of doc.content ?? []) {
    if (block.type === "paragraph") {
      for (const seg of inlineNodesToSegments(block.content ?? [])) {
        segments.push(seg);
      }
    } else if (block.type === "bulletList" || block.type === "orderedList") {
      const listSeg: ListSegment = {
        kind: "list",
        order_type: block.type === "bulletList" ? "unordered" : "lowercase_alphabetic",
        order_format: block.type === "bulletList" ? "none" : "parenthesis",
        items: (block.content ?? []).map((listItem) => {
          // Each listItem has content: [paragraph, ...]
          const inlineNodes = (listItem.content ?? []).flatMap(
            (para) => (para.content ?? []) as JSONContent[],
          );
          return inlineNodesToSegments(inlineNodes);
        }),
      };
      segments.push(listSeg);
    }
  }

  return segments;
}

// ── Editor setup ──────────────────────────────────────────────────────────────

const editor = useEditor({
  extensions: [
    StarterKit.configure({
      // Disable heading, blockquote, codeBlock — problem content is inline
      heading: false,
      blockquote: false,
      codeBlock: false,
      horizontalRule: false,
    }),
    Underline,
    Superscript,
    Subscript,
    TextStyle,
    Color,
    Image.configure({ inline: true, allowBase64: true }),
  ],
  editorProps: {
    attributes: {
      class: "pe-editor-body",
      spellcheck: "false",
    },
    handlePaste(view, event) {
      const items = event.clipboardData?.items;
      if (!items) return false;
      for (const item of Array.from(items)) {
        if (item.type.startsWith("image/")) {
          event.preventDefault();
          const file = item.getAsFile();
          if (!file) continue;
          const reader = new FileReader();
          reader.onload = (e) => {
            const dataUri = e.target?.result as string;
            if (!dataUri) return;
            const imageNode = view.state.schema.nodes["image"]?.create({
              src: dataUri,
              alt: "",
            });
            if (imageNode) {
              view.dispatch(view.state.tr.replaceSelectionWith(imageNode));
            }
          };
          reader.readAsDataURL(file);
          return true;
        }
      }
      return false;
    },
  },
});

// ── Load content ──────────────────────────────────────────────────────────────

async function loadContent() {
  if (!editor.value) return;
  loading.value = true;
  error.value = "";
  try {
    const resp = await getProblemContent(props.problemId);
    questionId.value = resp.question_id;
    const doc = segmentsToDoc(resp.content);
    editor.value.commands.setContent(doc);

    if (resp.choice_pool) {
      choices.value = resp.choice_pool.choices.map((c) =>
        c.content
          .filter((s): s is TextSegment => s.kind === "text")
          .map((s) => s.text)
          .join(""),
      );
      orderType.value = resp.choice_pool.order_type;
      orderFormat.value = resp.choice_pool.order_format;
    } else {
      choices.value = [];
      orderType.value = "uppercase_alphabetic";
      orderFormat.value = "parenthesis";
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(loadContent);
watch(() => props.problemId, loadContent);

onBeforeUnmount(() => editor.value?.destroy());

// ── Save ──────────────────────────────────────────────────────────────────────

async function handleSave() {
  if (!editor.value) return;
  saving.value = true;
  error.value = "";
  try {
    const doc = editor.value.getJSON();
    const segments = docToSegments(doc);

    const choicePool: ChoicePoolDto | null =
      choices.value.length > 0
        ? {
            order_type: orderType.value,
            order_format: orderFormat.value,
            choices: choices.value.map((text) => ({
              content: [
                {
                  kind: "text" as const,
                  text,
                  bold: false,
                  italic: false,
                  underline: false,
                  underwave: false,
                  strikethrough: false,
                  superscript: false,
                  subscript: false,
                  monospace: false,
                  formula: false,
                  red: false,
                },
              ],
            })),
          }
        : null;

    await saveProblemContent(props.problemId, segments, choicePool);
    emit("saved");
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

// ── Image insert ──────────────────────────────────────────────────────────────

function triggerImagePicker() {
  imageInput.value?.click();
}

function onImageFileSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file || !editor.value) return;

  const reader = new FileReader();
  reader.onload = (e) => {
    const dataUri = e.target?.result as string;
    if (dataUri) {
      editor.value!.chain().focus().setImage({ src: dataUri }).run();
    }
  };
  reader.readAsDataURL(file);
  // Reset so the same file can be picked again
  input.value = "";
}

// ── Toolbar helpers ───────────────────────────────────────────────────────────

function toggleBold() {
  editor.value?.chain().focus().toggleBold().run();
}
function toggleItalic() {
  editor.value?.chain().focus().toggleItalic().run();
}
function toggleUnderline() {
  editor.value?.chain().focus().toggleUnderline().run();
}
function toggleStrike() {
  editor.value?.chain().focus().toggleStrike().run();
}
function toggleSuperscript() {
  editor.value?.chain().focus().toggleSuperscript().run();
}
function toggleSubscript() {
  editor.value?.chain().focus().toggleSubscript().run();
}
function toggleCode() {
  editor.value?.chain().focus().toggleCode().run();
}
function toggleBulletList() {
  editor.value?.chain().focus().toggleBulletList().run();
}
function toggleOrderedList() {
  editor.value?.chain().focus().toggleOrderedList().run();
}
function toggleRed() {
  if (!editor.value) return;
  if (editor.value.isActive("textStyle", { color: "#e53e3e" })) {
    editor.value.chain().focus().unsetColor().run();
  } else {
    editor.value.chain().focus().setColor("#e53e3e").run();
  }
}
function undo() {
  editor.value?.chain().focus().undo().run();
}
function redo() {
  editor.value?.chain().focus().redo().run();
}

function isActive(name: string, attrs?: Record<string, unknown>) {
  return editor.value?.isActive(name, attrs) ?? false;
}
</script>

<template>
  <div class="problem-editor">
    <!-- Hidden file input for image insertion -->
    <input
      ref="imageInput"
      type="file"
      accept="image/png,image/jpeg,image/gif,image/webp,image/svg+xml"
      style="display: none"
      @change="onImageFileSelected"
    />

    <!-- Toolbar -->
    <div class="pe-toolbar" v-if="!loading">
      <div class="pe-toolbar-group">
        <button
          class="pe-btn"
          :class="{ active: isActive('bold') }"
          title="Bold (Ctrl+B)"
          @click="toggleBold"
        >
          <strong>B</strong>
        </button>
        <button
          class="pe-btn"
          :class="{ active: isActive('italic') }"
          title="Italic (Ctrl+I)"
          @click="toggleItalic"
        >
          <em>I</em>
        </button>
        <button
          class="pe-btn"
          :class="{ active: isActive('underline') }"
          title="Underline (Ctrl+U)"
          @click="toggleUnderline"
        >
          <span style="text-decoration: underline">U</span>
        </button>
        <button
          class="pe-btn"
          :class="{ active: isActive('strike') }"
          title="Strikethrough"
          @click="toggleStrike"
        >
          <span style="text-decoration: line-through">S</span>
        </button>
      </div>

      <div class="pe-toolbar-sep" />

      <div class="pe-toolbar-group">
        <button
          class="pe-btn"
          :class="{ active: isActive('superscript') }"
          title="Superscript"
          @click="toggleSuperscript"
        >
          x<sup>2</sup>
        </button>
        <button
          class="pe-btn"
          :class="{ active: isActive('subscript') }"
          title="Subscript"
          @click="toggleSubscript"
        >
          x<sub>2</sub>
        </button>
        <button
          class="pe-btn"
          :class="{ active: isActive('code') }"
          title="Monospace / Formula"
          @click="toggleCode"
        >
          <code>{ }</code>
        </button>
      </div>

      <div class="pe-toolbar-sep" />

      <div class="pe-toolbar-group">
        <button
          class="pe-btn"
          :class="{ active: isActive('bulletList') }"
          title="Bullet list"
          @click="toggleBulletList"
        >
          &#8226;&#8212;
        </button>
        <button
          class="pe-btn"
          :class="{ active: isActive('orderedList') }"
          title="Ordered list"
          @click="toggleOrderedList"
        >
          1&#8212;
        </button>
      </div>

      <div class="pe-toolbar-sep" />

      <div class="pe-toolbar-group">
        <button
          class="pe-btn pe-btn--red"
          :class="{ active: isActive('textStyle', { color: '#e53e3e' }) }"
          title="Red text"
          @click="toggleRed"
        >
          <span style="color: #e53e3e; font-weight: 700">A</span>
        </button>
        <button class="pe-btn" title="Insert image" @click="triggerImagePicker">🖼</button>
      </div>

      <div class="pe-toolbar-sep" />

      <div class="pe-toolbar-group">
        <button
          class="pe-btn"
          title="Undo (Ctrl+Z)"
          :disabled="!editor?.can().undo()"
          @click="undo"
        >
          ↩
        </button>
        <button
          class="pe-btn"
          title="Redo (Ctrl+Shift+Z)"
          :disabled="!editor?.can().redo()"
          @click="redo"
        >
          ↪
        </button>
      </div>

      <div class="pe-toolbar-spacer" />

      <div class="pe-toolbar-group pe-actions">
        <span v-if="error" class="pe-error-inline">{{ error }}</span>
        <button class="pe-btn pe-btn--ghost" @click="$emit('cancelled')">Cancel</button>
        <button class="pe-btn pe-btn--primary" :disabled="saving" @click="handleSave">
          {{ saving ? "Saving…" : "Save" }}
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="pe-state">Loading content…</div>
    <div v-else-if="error && !editor" class="pe-state pe-state--error">{{ error }}</div>

    <!-- Editor area -->
    <div v-show="!loading" class="pe-editor-wrap">
      <EditorContent :editor="editor" />
    </div>

    <!-- Options / choice pool -->
    <div v-if="!loading" class="pe-choices">
      <div class="pe-choices-header">
        <span class="pe-choices-title">Options</span>
        <div class="pe-choices-controls">
          <select v-model="orderType" class="pe-select" title="Order type">
            <option value="uppercase_alphabetic">A, B, C</option>
            <option value="lowercase_alphabetic">a, b, c</option>
            <option value="decimal">1, 2, 3</option>
            <option value="uppercase_roman">I, II, III</option>
            <option value="lowercase_roman">i, ii, iii</option>
            <option value="unordered">• Bullet</option>
          </select>
          <select v-model="orderFormat" class="pe-select" title="Order format">
            <option value="parenthesis">(A)</option>
            <option value="right_parenthesis">A)</option>
            <option value="period">A.</option>
            <option value="none">A</option>
          </select>
          <button class="pe-btn pe-btn--add" @click="addChoice" title="Add option">+ Add</button>
        </div>
      </div>

      <div v-if="choices.length === 0" class="pe-choices-empty">
        No options — click + Add to create a multiple-choice question.
      </div>

      <div v-else class="pe-choices-list">
        <div v-for="(choice, i) in choices" :key="i" class="pe-choice-row">
          <span class="pe-choice-label">{{ i + 1 }}</span>
          <input
            v-model="choices[i]"
            class="pe-choice-input"
            type="text"
            placeholder="Option text…"
          />
          <button
            class="pe-choice-btn"
            :disabled="i === 0"
            @click="moveChoice(i, -1)"
            title="Move up"
          >
            ↑
          </button>
          <button
            class="pe-choice-btn"
            :disabled="i === choices.length - 1"
            @click="moveChoice(i, 1)"
            title="Move down"
          >
            ↓
          </button>
          <button class="pe-choice-btn pe-choice-btn--del" @click="removeChoice(i)" title="Remove">
            ✕
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.problem-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  font-family:
    system-ui,
    -apple-system,
    sans-serif;
}

/* ── Toolbar ───────────────────────────────────────────────────────── */

.pe-toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 2px;
  padding: 0.4rem 0.6rem;
  background: #f0f2f8;
  border-bottom: 1px solid #d8dce8;
  flex-shrink: 0;
}

.pe-toolbar-group {
  display: flex;
  align-items: center;
  gap: 1px;
}

.pe-toolbar-sep {
  width: 1px;
  height: 20px;
  background: #ccd0de;
  margin: 0 4px;
}

.pe-toolbar-spacer {
  flex: 1;
}

.pe-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 28px;
  min-width: 28px;
  padding: 0 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 0.82rem;
  color: #333;
  transition:
    background 0.1s,
    border-color 0.1s;
  user-select: none;
}

.pe-btn:hover:not(:disabled) {
  background: #e2e6f0;
  border-color: #c5cad8;
}

.pe-btn.active {
  background: #d0d9f8;
  border-color: #8098e0;
  color: #1a3494;
}

.pe-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.pe-btn--primary {
  background: #2563eb;
  color: #fff;
  border-color: #1d4ed8;
  font-weight: 600;
  padding: 0 12px;
}

.pe-btn--primary:hover:not(:disabled) {
  background: #1d4ed8;
  border-color: #1e40af;
}

.pe-btn--ghost {
  color: #666;
  padding: 0 10px;
}

.pe-actions {
  gap: 6px;
}

.pe-error-inline {
  font-size: 0.78rem;
  color: #c0392b;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Editor body ───────────────────────────────────────────────────── */

.pe-editor-wrap {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem 2rem;
  background: #fff;
}

:deep(.pe-editor-body) {
  font-family: "Georgia", "Times New Roman", serif;
  font-size: 1rem;
  line-height: 1.7;
  color: #1a1a1a;
  outline: none;
  min-height: 120px;
}

:deep(.pe-editor-body p) {
  margin: 0.5rem 0;
}

:deep(.pe-editor-body strong) {
  font-weight: 700;
}

:deep(.pe-editor-body em) {
  font-style: italic;
}

:deep(.pe-editor-body u) {
  text-decoration: underline;
}

:deep(.pe-editor-body s) {
  text-decoration: line-through;
}

:deep(.pe-editor-body sup) {
  font-size: 0.72em;
  vertical-align: super;
}

:deep(.pe-editor-body sub) {
  font-size: 0.72em;
  vertical-align: sub;
}

:deep(.pe-editor-body code) {
  font-family: "Courier New", Courier, monospace;
  background: #f4f4f4;
  padding: 0 3px;
  border-radius: 3px;
  font-size: 0.9em;
}

:deep(.pe-editor-body img) {
  max-width: 100%;
  height: auto;
  display: inline-block;
  vertical-align: middle;
  border-radius: 4px;
  margin: 4px 0;
  cursor: default;
}

:deep(.pe-editor-body ul) {
  padding-left: 1.5rem;
  margin: 0.5rem 0;
}

:deep(.pe-editor-body ol) {
  padding-left: 1.75rem;
  margin: 0.5rem 0;
}

:deep(.pe-editor-body .ProseMirror-selectednode) {
  outline: 2px solid #2563eb;
  border-radius: 2px;
}

/* ── States ────────────────────────────────────────────────────────── */

.pe-state {
  color: #888;
  font-size: 0.9rem;
  padding: 1.5rem;
}

.pe-state--error {
  color: #c0392b;
  background: #fff5f5;
  border-radius: 6px;
  margin: 1rem;
}

/* ── Options / choice pool ──────────────────────────────────────────── */

.pe-choices {
  flex-shrink: 0;
  border-top: 1px solid #e0e4ee;
  background: #f8f9fd;
  max-height: 280px;
  overflow-y: auto;
}

.pe-choices-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.45rem 0.75rem;
  border-bottom: 1px solid #e4e8f2;
  background: #f0f2f8;
}

.pe-choices-title {
  font-size: 0.72rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #666;
}

.pe-choices-controls {
  display: flex;
  align-items: center;
  gap: 6px;
}

.pe-select {
  height: 26px;
  font-size: 0.78rem;
  padding: 0 6px;
  border: 1px solid #c8ccda;
  border-radius: 4px;
  background: #fff;
  color: #333;
  cursor: pointer;
}

.pe-btn--add {
  font-size: 0.78rem;
  padding: 0 10px;
  height: 26px;
  background: #2563eb;
  color: #fff;
  border-color: #1d4ed8;
  font-weight: 600;
}

.pe-btn--add:hover:not(:disabled) {
  background: #1d4ed8;
}

.pe-choices-empty {
  font-size: 0.8rem;
  color: #aaa;
  padding: 0.75rem 1rem;
  font-style: italic;
}

.pe-choices-list {
  padding: 0.4rem 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pe-choice-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.pe-choice-label {
  font-size: 0.72rem;
  color: #999;
  min-width: 16px;
  text-align: right;
  user-select: none;
}

.pe-choice-input {
  flex: 1;
  height: 28px;
  font-size: 0.85rem;
  padding: 0 8px;
  border: 1px solid #d0d4e0;
  border-radius: 4px;
  background: #fff;
  color: #1a1a1a;
  font-family: inherit;
}

.pe-choice-input:focus {
  outline: none;
  border-color: #2563eb;
  box-shadow: 0 0 0 2px #dbeafe;
}

.pe-choice-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 24px;
  min-width: 24px;
  padding: 0 4px;
  font-size: 0.75rem;
  border: 1px solid #ccc;
  border-radius: 3px;
  background: #fff;
  color: #555;
  cursor: pointer;
  transition: background 0.1s;
}

.pe-choice-btn:hover:not(:disabled) {
  background: #e8eaf0;
}

.pe-choice-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.pe-choice-btn--del {
  color: #c0392b;
  border-color: #f5c6c6;
}

.pe-choice-btn--del:hover:not(:disabled) {
  background: #fff5f5;
  border-color: #e57373;
}
</style>
