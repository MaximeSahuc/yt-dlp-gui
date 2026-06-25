<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import IconMdiAlertCircle from "~icons/mdi/alert-circle";
import IconMdiFolderAlertOutline from "~icons/mdi/folder-alert-outline";
import IconMdiCheckCircle from "~icons/mdi/check-circle";
import IconMdiDownload from "~icons/mdi/download";
import IconMdiFolderOpenOutline from "~icons/mdi/folder-open-outline";
import { useSettingStore } from "@/stores/setting";
import type { YtdlpStatus, DenoStatus, DownloadProgress } from "@/types";

const { t } = useI18n();
const settingStore = useSettingStore();

const checking = ref(true);
const ytdlpInstalled = ref(false);
const denoInstalled = ref(false);

const downloading = ref(false);
const downloadLabel = ref("");
const downloadPercent = ref(0);

const showReady = ref(false);
let readyTimer: ReturnType<typeof setTimeout> | null = null;

const toolsReady = computed(() => ytdlpInstalled.value && denoInstalled.value);
const folderSet = computed(() => !!settingStore.downloadDir);
const allReady = computed(() => toolsReady.value && folderSet.value);

async function chooseFolder() {
  const result = await open({ directory: true, multiple: false });
  if (typeof result === "string" && result) {
    settingStore.downloadDir = result;
  }
}

async function checkStatus() {
  try {
    const [yt, deno] = await Promise.all([
      invoke<YtdlpStatus>("get_ytdlp_status"),
      invoke<DenoStatus>("get_deno_status"),
    ]);
    ytdlpInstalled.value = !!yt.installed;
    denoInstalled.value = !!deno.installed;
  } catch {
    // 检测失败时按未安装处理，让用户可以重试下载
  } finally {
    checking.value = false;
  }
}

function flashReady() {
  showReady.value = true;
  if (readyTimer) clearTimeout(readyTimer);
  readyTimer = setTimeout(() => (showReady.value = false), 4000);
}

async function downloadOne(
  command: "download_ytdlp" | "download_deno",
  event: "ytdlp-download-progress" | "deno-download-progress",
  label: string,
) {
  downloadLabel.value = label;
  downloadPercent.value = 0;
  const unlisten = await listen<DownloadProgress>(event, (e) => {
    downloadPercent.value = Math.round(e.payload.percent);
  });
  try {
    await invoke(command);
  } finally {
    unlisten();
  }
}

async function downloadMissing() {
  downloading.value = true;
  try {
    if (!ytdlpInstalled.value) {
      await downloadOne("download_ytdlp", "ytdlp-download-progress", t("mp3buddy.toolYtdlp"));
    }
    if (!denoInstalled.value) {
      await downloadOne("download_deno", "deno-download-progress", t("mp3buddy.toolDeno"));
    }
    await checkStatus();
    if (allReady.value) flashReady();
  } catch (e: unknown) {
    window.$message?.error(t("common.downloadFailed", { e }));
  } finally {
    downloading.value = false;
  }
}

// 当一切就绪（含输出目录），短暂提示用户可以开始
watch(allReady, (now, prev) => {
  if (now && !prev) flashReady();
});

onMounted(async () => {
  await checkStatus();
  if (allReady.value) flashReady();
});

onBeforeUnmount(() => {
  if (readyTimer) clearTimeout(readyTimer);
});
</script>

<template>
  <!-- 缺少工具：提示 + 下载按钮 -->
  <div v-if="!checking && !toolsReady" class="setup-banner setup-banner--warn">
    <div class="setup-banner-main">
      <n-icon size="18" color="#f0a020" class="setup-icon">
        <IconMdiAlertCircle />
      </n-icon>
      <div class="setup-text">
        <div class="setup-title">{{ t("mp3buddy.setupTitle") }}</div>
        <div class="setup-list">
          <span v-if="!ytdlpInstalled" class="setup-pill">{{ t("mp3buddy.toolYtdlp") }}</span>
          <span v-if="!denoInstalled" class="setup-pill">{{ t("mp3buddy.toolDeno") }}</span>
        </div>
      </div>
      <n-button
        type="primary"
        size="small"
        :loading="downloading"
        :disabled="downloading"
        @click="downloadMissing"
      >
        <template #icon>
          <n-icon><IconMdiDownload /></n-icon>
        </template>
        {{ t("common.download") }}
      </n-button>
    </div>
    <div v-if="downloading" class="setup-progress">
      <span class="setup-progress-label">
        {{ t("mp3buddy.setupDownloading", { name: downloadLabel }) }}
      </span>
      <n-progress
        type="line"
        :percentage="downloadPercent"
        :show-indicator="false"
        :height="4"
        :processing="true"
      />
    </div>
  </div>

  <!-- 工具就绪但未设置输出目录 -->
  <div v-else-if="!checking && !folderSet" class="setup-banner setup-banner--warn">
    <div class="setup-banner-main">
      <n-icon size="18" color="#f0a020" class="setup-icon">
        <IconMdiFolderAlertOutline />
      </n-icon>
      <div class="setup-text">
        <div class="setup-title">{{ t("mp3buddy.setupNoFolder") }}</div>
      </div>
      <n-button type="primary" size="small" @click="chooseFolder">
        <template #icon>
          <n-icon><IconMdiFolderOpenOutline /></n-icon>
        </template>
        {{ t("mp3buddy.setupChooseFolder") }}
      </n-button>
    </div>
  </div>

  <!-- 全部就绪：短暂提示 -->
  <div v-else-if="showReady" class="setup-banner setup-banner--ok">
    <n-icon size="18" color="#18a058" class="setup-icon">
      <IconMdiCheckCircle />
    </n-icon>
    <span class="setup-ready-text">{{ t("mp3buddy.setupReady") }}</span>
  </div>
</template>

<style scoped lang="scss">
.setup-banner {
  flex-shrink: 0;
  padding: 10px 14px;
  border-bottom: 1px solid var(--mp3-border);
}

.setup-banner--warn {
  background: var(--mp3-surface-2);
}

.setup-banner--ok {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--mp3-surface-2);
}

.setup-banner-main {
  display: flex;
  align-items: center;
  gap: 10px;
}

.setup-icon {
  flex-shrink: 0;
}

.setup-text {
  flex: 1;
  min-width: 0;
}

.setup-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--mp3-text);
}

.setup-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
}

.setup-pill {
  font-size: 11px;
  color: var(--mp3-text-3);
  background: var(--mp3-surface-3);
  border: 1px solid var(--mp3-border);
  border-radius: 10px;
  padding: 1px 8px;
}

.setup-progress {
  margin-top: 8px;
}

.setup-progress-label {
  display: block;
  font-size: 11px;
  color: var(--mp3-text-3);
  margin-bottom: 4px;
}

.setup-ready-text {
  font-size: 12px;
  font-weight: 600;
  color: #18a058;
}
</style>
