<script setup lang="ts">
import { useI18n } from "vue-i18n";
import IconMdiImageOutline from "~icons/mdi/image-outline";
import IconMdiAccount from "~icons/mdi/account";
import IconMdiCheckCircle from "~icons/mdi/check-circle";
import IconMdiAlertCircle from "~icons/mdi/alert-circle";
import type { VideoInfo } from "@/types";

defineProps<{
  state: "idle" | "loading" | "ready" | "error";
  info: VideoInfo | null;
  errorMsg?: string;
}>();

const { t } = useI18n();

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${m}:${String(s).padStart(2, "0")}`;
}
</script>

<template>
  <div class="preview-box">
    <div v-if="state === 'idle'" class="preview-center">
      <n-icon size="44" color="var(--mp3-icon-faint)">
        <IconMdiImageOutline />
      </n-icon>
      <p class="preview-hint">{{ t("mp3buddy.idlePlaceholder") }}</p>
    </div>

    <div v-else-if="state === 'loading'" class="preview-center">
      <n-spin size="medium" />
      <p class="preview-hint">{{ t("mp3buddy.fetching") }}</p>
    </div>

    <div v-else-if="state === 'error'" class="preview-center">
      <n-icon size="36" color="#d03050">
        <IconMdiAlertCircle />
      </n-icon>
      <p class="preview-hint error-text">{{ errorMsg || t("mp3buddy.errorFetch") }}</p>
    </div>

    <template v-else-if="state === 'ready' && info">
      <div class="preview-thumb-wrap">
        <img :src="info.thumbnail" alt="" class="preview-thumb" />
      </div>
      <div class="preview-meta">
        <div class="preview-title">{{ info.title }}</div>
        <div class="preview-row">
          <n-icon size="13" color="var(--mp3-text-3)" style="flex-shrink:0">
            <IconMdiAccount />
          </n-icon>
          <span class="preview-sub">{{ info.uploader }}</span>
        </div>
        <div class="preview-row">
          <n-icon size="13" color="#18a058" style="flex-shrink:0">
            <IconMdiCheckCircle />
          </n-icon>
          <span class="preview-sub ready-text">{{ t("mp3buddy.ready") }}</span>
          <span v-if="info.duration" class="preview-sub duration">
            {{ formatDuration(info.duration) }}
          </span>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped lang="scss">
.preview-box {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.preview-center {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 16px;
}

.preview-hint {
  font-size: 12px;
  color: var(--mp3-text-muted);
  text-align: center;
  margin: 0;
}

.error-text {
  color: #d03050;
}

.preview-thumb-wrap {
  width: 100%;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  flex-shrink: 0;
}

.preview-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.preview-meta {
  padding: 8px 2px 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex-shrink: 0;
}

.preview-title {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.35;
  color: var(--mp3-text);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.preview-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

.preview-sub {
  font-size: 12px;
  color: var(--mp3-text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ready-text {
  color: #18a058;
  font-weight: 600;
}

.duration {
  color: var(--mp3-text-muted);
  margin-left: 4px;
}
</style>
