<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { NButton, NIcon } from "naive-ui";
import IconMdiClipboardOutline from "~icons/mdi/clipboard-outline";
import IconMdiDownload from "~icons/mdi/download";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { isValidUrl } from "@/utils/validate";

const props = defineProps<{
  modelValue: string;
  state: "idle" | "loading" | "ready" | "error";
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  download: [payload: { quality: string; format: string }];
}>();

const { t } = useI18n();
const quality = ref("320K");
const format = ref("mp3");

const qualityOptions = computed(() => [
  { label: t("mp3buddy.quality320"), value: "320K" },
  { label: t("mp3buddy.quality256"), value: "256K" },
  { label: t("mp3buddy.quality192"), value: "192K" },
  { label: t("mp3buddy.quality128"), value: "128K" },
]);

const formatOptions = [
  { label: "MP3", value: "mp3" },
  { label: "WAV", value: "wav" },
  { label: "Opus", value: "opus" },
];

function startDownload() {
  emit("download", { quality: quality.value, format: format.value });
}

async function handlePaste() {
  try {
    const text = await readText();
    const trimmed = text?.trim() ?? "";
    if (trimmed && isValidUrl(trimmed)) {
      emit("update:modelValue", trimmed);
    } else {
      window.$message?.warning(t("clipboard.invalidUrl"));
    }
  } catch {
    window.$message?.warning(t("clipboard.readFailed"));
  }
}

function onInput(e: Event) {
  emit("update:modelValue", (e.target as HTMLInputElement).value);
}
</script>

<template>
  <div class="dl-panel">
    <!-- Row 1: URL + Paste -->
    <div class="url-row">
      <input
        class="url-input"
        type="text"
        :value="props.modelValue"
        :placeholder="t('mp3buddy.urlPlaceholder')"
        @input="onInput"
        @keydown.enter="startDownload"
      />
      <button class="paste-btn" type="button" @click="handlePaste">
        <NIcon size="14"><IconMdiClipboardOutline /></NIcon>
        {{ t("mp3buddy.paste") }}
      </button>
    </div>

    <!-- Row 2: Quality select + Format select + Download button -->
    <div class="action-row">
      <n-select
        v-model:value="quality"
        :options="qualityOptions"
        :consistent-menu-width="false"
        class="quality-select"
      />
      <n-select
        v-model:value="format"
        :options="formatOptions"
        :consistent-menu-width="false"
        class="format-select"
      />
      <NButton
        type="primary"
        :disabled="props.state !== 'ready'"
        class="dl-btn"
        @click="startDownload"
      >
        <template #icon>
          <NIcon><IconMdiDownload /></NIcon>
        </template>
        {{ t("mp3buddy.download") }}
      </NButton>
    </div>
  </div>
</template>

<style scoped lang="scss">
.dl-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 10px;
  border-top: 1px solid var(--mp3-border);
  flex-shrink: 0;
}

.url-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.url-input {
  flex: 1;
  height: 34px;
  border: 1px solid var(--mp3-border-strong);
  border-radius: 6px;
  padding: 0 10px;
  font-size: 13px;
  background: var(--mp3-input-bg);
  color: var(--mp3-text);
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.2s;

  &::placeholder {
    color: var(--mp3-text-muted);
  }

  &:focus {
    border-color: var(--mp3-accent);
    box-shadow: 0 0 0 2px rgba(74, 156, 214, 0.15);
  }
}

.paste-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 34px;
  padding: 0 12px;
  border: 1px solid var(--mp3-border-strong);
  border-radius: 6px;
  background: var(--mp3-surface-3);
  font-size: 13px;
  color: var(--mp3-text-2);
  cursor: pointer;
  white-space: nowrap;
  box-sizing: border-box;
  transition: background 0.15s, border-color 0.15s;

  &:hover {
    background: var(--mp3-hover-soft);
    border-color: var(--mp3-text-muted);
  }
}

.action-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.quality-select {
  flex: 1;
}

.format-select {
  width: 110px;
  flex-shrink: 0;
}

.dl-btn {
  background: #18a058 !important;
  border: none !important;
  color: #fff !important;
  font-weight: 700;
  white-space: nowrap;
}
</style>
