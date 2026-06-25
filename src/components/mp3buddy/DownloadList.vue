<script setup lang="ts">
import { useI18n } from "vue-i18n";
import IconMdiCircle from "~icons/mdi/circle";
import IconMdiFolderOpenOutline from "~icons/mdi/folder-open-outline";
import { open } from "@tauri-apps/plugin-dialog";
import { useDownloadStore } from "@/stores/download";
import { useSettingStore } from "@/stores/setting";

const { t } = useI18n();
const downloadStore = useDownloadStore();
const settingStore = useSettingStore();

function isActive(status: string): boolean {
  return status === "downloading" || status === "queued" || status === "paused";
}

function statusColor(status: string): string {
  if (status === "completed") return "#18a058";
  if (status === "error") return "#d03050";
  return "#f0a020";
}

function truncatePath(p: string): string {
  if (!p) return "";
  if (p.length <= 28) return p;
  return "…" + p.slice(-25);
}

async function changeFolder() {
  const result = await open({ directory: true, multiple: false });
  if (typeof result === "string" && result) {
    settingStore.downloadDir = result;
  }
}
</script>

<template>
  <div class="dl-list">
    <div class="dl-list-title">{{ t("mp3buddy.downloadsTitle") }}</div>
    <div class="dl-items">
      <div v-if="downloadStore.tasks.length === 0" class="dl-empty">
        <span class="dl-empty-text">–</span>
      </div>
      <div v-for="task in downloadStore.tasks" :key="task.id" class="dl-item">
        <div class="dl-item-row">
          <n-icon size="10" :color="statusColor(task.status)" style="flex-shrink:0; margin-top:2px">
            <IconMdiCircle />
          </n-icon>
          <div class="dl-item-info">
            <div class="dl-item-title">{{ task.title }}</div>
            <div v-if="task.uploader" class="dl-item-uploader">{{ task.uploader }}</div>
            <div class="dl-item-label">{{ task.formatLabel }}</div>
          </div>
        </div>
        <n-progress
          v-if="isActive(task.status)"
          type="line"
          :percentage="task.percent"
          :status="task.status === 'paused' ? 'warning' : 'default'"
          :processing="task.status === 'downloading'"
          :show-indicator="false"
          :height="3"
          style="margin-top: 4px"
        />
        <div v-if="isActive(task.status)" class="dl-item-percent">
          <template v-if="task.status === 'queued'">{{ t("downloads.status.queued") }}</template>
          <template v-else-if="task.status === 'paused'">{{ t("downloads.status.paused") }}</template>
          <template v-else-if="task.percent > 0 || task.speed">
            {{ task.percent }}%<span v-if="task.speed"> · {{ task.speed }}</span>
          </template>
          <template v-else>{{ t("mp3buddy.preparing") }}</template>
        </div>
      </div>
    </div>
    <div class="dl-footer">
      <div class="dl-folder-row">
        <n-icon size="13" color="var(--mp3-text-3)" style="flex-shrink: 0">
          <IconMdiFolderOpenOutline />
        </n-icon>
        <span class="dl-folder-path" :title="settingStore.downloadDir || ''">
          {{ settingStore.downloadDir ? truncatePath(settingStore.downloadDir) : t("mp3buddy.noFolder") }}
        </span>
      </div>
      <button class="folder-btn" type="button" @click="changeFolder">
        {{ t("mp3buddy.changeFolder") }}
      </button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.dl-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.dl-list-title {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: var(--mp3-text-3);
  padding: 8px 10px 4px;
  border-bottom: 1px solid var(--mp3-border);
  flex-shrink: 0;
}

.dl-items {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.dl-empty {
  padding: 20px 10px;
  text-align: center;
}

.dl-empty-text {
  font-size: 11px;
  color: var(--mp3-text-muted);
}

.dl-item {
  padding: 5px 10px;
  border-bottom: 1px solid var(--mp3-divider-soft);
}

.dl-item-row {
  display: flex;
  align-items: flex-start;
  gap: 5px;
}

.dl-item-info {
  flex: 1;
  min-width: 0;
}

.dl-item-title {
  font-size: 11px;
  font-weight: 600;
  line-height: 1.3;
  color: var(--mp3-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dl-item-uploader {
  font-size: 10px;
  color: var(--mp3-text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dl-item-label {
  font-size: 10px;
  color: var(--mp3-text-muted);
}

.dl-item-percent {
  font-size: 10px;
  color: var(--mp3-text-muted);
  margin-top: 1px;
}

.dl-footer {
  flex-shrink: 0;
  padding: 8px 10px;
  border-top: 1px solid var(--mp3-border);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dl-folder-row {
  display: flex;
  align-items: center;
  gap: 4px;
}


.dl-folder-path {
  font-size: 10px;
  color: var(--mp3-text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.folder-btn {
  width: 100%;
  height: 26px;
  border: 1px solid var(--mp3-border-strong);
  border-radius: 5px;
  background: var(--mp3-surface-3);
  font-size: 11px;
  color: var(--mp3-text-2);
  cursor: pointer;
  box-sizing: border-box;
  transition: background 0.15s, border-color 0.15s;

  &:hover {
    background: var(--mp3-hover);
    border-color: var(--mp3-text-muted);
  }
}
</style>
