<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { useVideoStore } from "@/stores/video";
import { useDownloadStore } from "@/stores/download";
import { useSettingStore } from "@/stores/setting";
import { isValidUrl } from "@/utils/validate";
import AppHeader from "@/components/mp3buddy/AppHeader.vue";
import DownloadList from "@/components/mp3buddy/DownloadList.vue";
import VideoPreview from "@/components/mp3buddy/VideoPreview.vue";
import DownloadPanel from "@/components/mp3buddy/DownloadPanel.vue";

const { t } = useI18n();
const route = useRoute();
const videoStore = useVideoStore();
const downloadStore = useDownloadStore();
const settingStore = useSettingStore();

const url = ref("");
const state = ref<"idle" | "loading" | "ready" | "error">("idle");
const preview = ref<import("@/types").VideoInfo | null>(null);
const errorMsg = ref("");

let fetchGen = 0;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(url, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  if (!val.trim()) {
    state.value = "idle";
    preview.value = null;
    return;
  }
  if (!isValidUrl(val.trim())) {
    state.value = "idle";
    preview.value = null;
    return;
  }
  debounceTimer = setTimeout(() => fetchInfo(val.trim()), 400);
});

async function fetchInfo(targetUrl: string) {
  const gen = ++fetchGen;
  state.value = "loading";
  preview.value = null;
  const data = await videoStore.fetchVideoInfo(targetUrl);
  if (gen !== fetchGen) return;
  if (data) {
    preview.value = data.videoInfo;
    state.value = "ready";
  } else {
    state.value = "error";
    errorMsg.value = t("mp3buddy.errorFetch");
  }
}

async function handleDownload(quality: string) {
  if (!settingStore.downloadDir) {
    window.$message?.warning(t("mp3buddy.noFolderWarning"));
    return;
  }
  if (state.value !== "ready" || !preview.value) return;

  const { cookieFile, cookieBrowser } = await videoStore.getCookieArgs();
  const id = Date.now().toString();
  const dlParams = {
    url: url.value,
    downloadDir: settingStore.downloadDir,
    downloadMode: "audio",
    videoFormat: null,
    audioFormat: "bestaudio",
    cookieFile,
    cookieBrowser,
    proxy: settingStore.proxy || null,
    outputTemplate: "%(title).200s.%(ext)s",
    concurrentFragments: settingStore.concurrentFragments || null,
    noOverwrites: settingStore.noOverwrites,
    embedSubs: false,
    embedThumbnail: false,
    embedMetadata: false,
    embedChapters: false,
    sponsorblockRemove: false,
    extractAudio: true,
    audioConvertFormat: "mp3",
    audioQuality: quality,
    noMerge: false,
    recodeFormat: null,
    limitRate: null,
    ffmpegArgs: null,
    subtitles: [],
    startTime: null,
    endTime: null,
    noPlaylist: false,
    playlistItems: null,
  };

  downloadStore.addTask({
    id,
    url: url.value,
    title: preview.value.title,
    thumbnail: preview.value.thumbnail ?? "",
    uploader: preview.value.uploader,
    formatLabel: `MP3 ${quality}`,
    status: "queued",
    percent: 0,
    speed: "",
    eta: "",
    downloaded: "",
    total: "",
    logs: [],
    createdAt: Date.now(),
    params: dlParams,
  });

  if (downloadStore.canStartNow()) {
    await invoke("start_download", { params: { id, ...dlParams } });
  }
}

onMounted(() => {
  const queryUrl = route.query.url;
  if (typeof queryUrl === "string" && queryUrl) {
    url.value = queryUrl;
  }
});
</script>

<template>
  <div class="mp3-page">
    <div class="mp3-card">
      <AppHeader />
      <div class="mp3-body">
        <div class="mp3-left">
          <DownloadList />
        </div>
        <div class="mp3-right">
          <VideoPreview :state="state" :info="preview" :error-msg="errorMsg" />
          <DownloadPanel v-model="url" :state="state" @download="handleDownload" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.mp3-page {
  width: 100%;
  height: calc(100vh - var(--titlebar-height));
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.mp3-card {
  width: 100%;
  height: 100%;
  background: var(--mp3-surface);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.mp3-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.mp3-left {
  width: 195px;
  min-width: 195px;
  flex-shrink: 0;
  border-right: 1px solid var(--mp3-divider);
  background: var(--mp3-surface-2);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.mp3-right {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  padding: 14px;
  gap: 0;
  overflow: hidden;
}
</style>
