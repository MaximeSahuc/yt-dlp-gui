<script setup lang="ts">
import type { YtdlpStatus, DenoStatus, DownloadProgress } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { check } from "@tauri-apps/plugin-updater";
import { useSettingStore } from "@/stores/setting";
import { useStatusStore } from "@/stores/status";
import { useI18n } from "vue-i18n";
import { localeEntries } from "@/locales";
import { getVersion } from "@tauri-apps/api/app";

const { t } = useI18n();
const router = useRouter();
const settingStore = useSettingStore();
const statusStore = useStatusStore();
const appVersion = ref("");

const platform = ref("");
const platformLabel = computed(() => {
  const map: Record<string, string> = {
    windows: "Windows",
    macos: "macOS",
    linux: "Linux",
  };
  return map[platform.value] || platform.value;
});

const localeOptions = localeEntries.map((e) => ({ label: `${e.flag} ${e.label}`, value: e.code }));

const themeModeOptions = computed(() => [
  { label: t("settings.themeAuto"), value: "auto" },
  { label: t("settings.themeLight"), value: "light" },
  { label: t("settings.themeDark"), value: "dark" },
]);

const concurrentFragmentsOptions = computed(() => [
  { label: t("settings.disabled"), value: 0 },
  { label: "2", value: 2 },
  { label: "4", value: 4 },
  { label: "8", value: 8 },
  { label: "16", value: 16 },
]);

const maxConcurrentOptions = computed(() => [
  { label: t("settings.unlimited"), value: 0 },
  { label: "1", value: 1 },
  { label: "2", value: 2 },
  { label: "3", value: 3 },
  { label: "5", value: 5 },
]);

const notifyModeOptions = computed(() => [
  { label: t("settings.noNotification"), value: "none" },
  { label: t("settings.inApp"), value: "app" },
  { label: t("settings.systemNotification"), value: "system" },
  { label: t("settings.all"), value: "all" },
]);

const applyYoutubeExtractorArgs = async () => {
  await invoke("set_youtube_extractor_args", {
    poToken: settingStore.youtubePoToken,
    visitorData: settingStore.youtubeVisitorData,
  });
};

const ytdlpStatus = ref<YtdlpStatus | null>(null);
const ytdlpChecking = ref(true);
const ytdlpDownloading = ref(false);
const ytdlpDownloadPercent = ref(0);
const ytdlpUpdating = ref(false);

const checkYtdlpStatus = async () => {
  ytdlpChecking.value = true;
  try {
    ytdlpStatus.value = await invoke<YtdlpStatus>("get_ytdlp_status");
  } catch (e) {
    // ignore
  } finally {
    ytdlpChecking.value = false;
  }
};

const handleDownloadYtdlp = async () => {
  ytdlpDownloading.value = true;
  ytdlpDownloadPercent.value = 0;
  const unlisten = await listen<DownloadProgress>("ytdlp-download-progress", (event) => {
    ytdlpDownloadPercent.value = event.payload.percent;
  });
  try {
    await invoke("download_ytdlp");
    window.$message.success(t("settings.ytdlpDownloadComplete"));
    await checkYtdlpStatus();
  } catch (e: unknown) {
    window.$message.error(t("common.downloadFailed", { e }));
  } finally {
    unlisten();
    ytdlpDownloading.value = false;
  }
};

const handleUpdateYtdlp = async () => {
  ytdlpUpdating.value = true;
  try {
    const result = await invoke<string>("update_ytdlp");
    if (result.includes("up to date")) {
      window.$message.success(t("settings.alreadyLatest"));
    } else if (result.includes("Updated")) {
      window.$message.success(t("settings.updatedToLatest"));
      await checkYtdlpStatus();
    } else {
      window.$message.success(t("settings.alreadyLatest"));
    }
  } catch (e: unknown) {
    window.$message.error(t("settings.updateFailed", { e }));
  } finally {
    ytdlpUpdating.value = false;
  }
};

const denoStatus = ref<DenoStatus | null>(null);
const denoChecking = ref(true);
const denoDownloading = ref(false);
const denoDownloadPercent = ref(0);

const checkDenoStatus = async () => {
  denoChecking.value = true;
  try {
    denoStatus.value = await invoke<DenoStatus>("get_deno_status");
  } catch (e) {
    // ignore
  } finally {
    denoChecking.value = false;
  }
};

const handleDownloadDeno = async () => {
  denoDownloading.value = true;
  denoDownloadPercent.value = 0;
  const unlisten = await listen<DownloadProgress>("deno-download-progress", (event) => {
    denoDownloadPercent.value = event.payload.percent;
  });
  try {
    await invoke("download_deno");
    window.$message.success(t("settings.denoDownloadComplete"));
    await checkDenoStatus();
  } catch (e: unknown) {
    window.$message.error(t("common.downloadFailed", { e }));
  } finally {
    unlisten();
    denoDownloading.value = false;
  }
};

const appUpdateChecking = ref(false);

const handleCheckAppUpdate = async () => {
  appUpdateChecking.value = true;
  try {
    const update = await check();
    if (update) {
      statusStore.updateVersion = update.version;
      statusStore.updateNotes = update.body || "";
      statusStore.showUpdateModal = true;
    } else {
      window.$message.success(t("settings.appIsLatest"));
    }
  } catch (e: unknown) {
    window.$message.error(t("settings.appUpdateFailed", { e }));
  } finally {
    appUpdateChecking.value = false;
  }
};

const refreshAll = () => {
  checkYtdlpStatus();
  checkDenoStatus();
};

const navSections = computed(() => [
  { id: "section-appearance", label: t("settings.appearance") },
  { id: "section-cookies", label: "Cookies" },
  { id: "section-dir", label: t("downloadDir.title") },
  { id: "section-advanced", label: t("settings.advanced") },
  { id: "section-about", label: t("settings.about") },
]);

/** 「高级」折叠面板的展开状态，默认关闭 */
const advancedExpanded = ref<string[]>([]);

const contentEl = ref<HTMLElement | null>(null);

function scrollToSection(id: string) {
  // 点击「高级」时先展开折叠面板，待内容撑开后再滚动定位
  if (id === "section-advanced" && !advancedExpanded.value.includes("advanced")) {
    advancedExpanded.value = ["advanced"];
  }
  nextTick(() => {
    const container = contentEl.value;
    const el = document.getElementById(id);
    if (!container || !el) return;
    const top = el.offsetTop - container.offsetTop;
    container.scrollTo({ top, behavior: "smooth" });
  });
}

onMounted(async () => {
  platform.value = await invoke<string>("get_platform");
  appVersion.value = await getVersion();
  await applyYoutubeExtractorArgs();
  refreshAll();
});

watch(
  () => [settingStore.youtubePoToken, settingStore.youtubeVisitorData],
  async () => {
    await applyYoutubeExtractorArgs();
  },
);
</script>

<template>
  <div class="settings-page">
    <!-- Header: same style as AppHeader -->
    <div class="settings-header">
      <div class="settings-header-left">
        <n-icon size="20" color="#fff">
          <icon-mdi-cog />
        </n-icon>
        <span class="settings-header-title">{{ $t("settings.title") }}</span>
      </div>
      <div class="settings-header-right">
        <n-button quaternary size="small" class="settings-btn" @click="refreshAll">
          <template #icon>
            <n-icon><icon-mdi-refresh /></n-icon>
          </template>
        </n-button>
        <n-button quaternary size="small" class="settings-btn" @click="router.push({ name: 'home' })">
          <template #icon>
            <n-icon><icon-mdi-home /></n-icon>
          </template>
        </n-button>
      </div>
    </div>

    <!-- Body -->
    <div class="settings-body">
      <!-- Left: section navigation -->
      <div class="settings-nav">
        <div class="settings-nav-title">{{ $t("settings.title") }}</div>
        <div class="settings-nav-items">
          <button
            v-for="section in navSections"
            :key="section.id"
            class="settings-nav-item"
            type="button"
            @click="scrollToSection(section.id)"
          >
            {{ section.label }}
          </button>
        </div>
      </div>

      <!-- Right: scrollable settings cards — forced light theme so it matches the main page -->
      <n-config-provider :theme="null" abstract>
      <div class="settings-content" ref="contentEl">

        <div id="section-appearance">
          <n-card :title="$t('settings.appearance')" size="small" class="section-card">
            <div class="info-list">
              <div class="info-row">
                <span class="info-label">{{ $t("settings.language") }}</span>
                <n-select
                  v-model:value="settingStore.locale"
                  :options="localeOptions"
                  style="width: 120px"
                  size="small"
                />
              </div>
              <div class="info-row">
                <span class="info-label">{{ $t("settings.themeMode") }}</span>
                <n-select
                  v-model:value="settingStore.themeMode"
                  :options="themeModeOptions"
                  style="width: 120px"
                  size="small"
                />
              </div>
              <div class="info-row">
                <span class="info-label">{{ $t("settings.autoCheckUpdate") }}</span>
                <n-switch v-model:value="settingStore.autoCheckUpdate" />
              </div>
            </div>
          </n-card>
        </div>

        <div id="section-cookies">
          <CookieCard class="section-card" />
        </div>

        <div id="section-dir">
          <DownloadDirCard class="section-card" />
        </div>

        <!-- 高级：默认折叠 -->
        <div id="section-advanced">
          <n-collapse v-model:expanded-names="advancedExpanded" class="advanced-collapse">
            <n-collapse-item :title="$t('settings.advanced')" name="advanced">
              <div id="section-ytdlp">
                <n-card title="yt-dlp" size="small" class="section-card">
                  <template #header-extra>
                    <n-flex align="center" :size="8">
                      <n-tag v-if="!ytdlpChecking" :type="ytdlpStatus?.installed ? 'success' : 'error'" round>
                        {{ ytdlpStatus?.installed ? $t("settings.installed") : $t("settings.notInstalled") }}
                      </n-tag>
                      <n-button
                        v-if="ytdlpStatus?.installed"
                        :loading="ytdlpUpdating"
                        strong
                        secondary
                        round
                        size="small"
                        @click="handleUpdateYtdlp"
                      >
                        {{ $t("settings.checkUpdate") }}
                      </n-button>
                      <n-button
                        v-if="ytdlpStatus && !ytdlpStatus.installed"
                        :loading="ytdlpDownloading"
                        :disabled="ytdlpDownloading"
                        type="primary"
                        size="small"
                        strong
                        secondary
                        round
                        @click="handleDownloadYtdlp"
                      >
                        {{ $t("common.download") }}
                      </n-button>
                    </n-flex>
                  </template>

                  <n-spin :show="ytdlpChecking">
                    <n-flex vertical :size="12">
                      <n-text depth="3" style="font-size: 13px">
                        {{ $t("settings.ytdlpDesc") }}
                      </n-text>

                      <n-alert
                        v-if="ytdlpStatus?.installed && ytdlpStatus.isManaged === false"
                        type="warning"
                        :bordered="false"
                        :show-icon="false"
                        style="font-size: 12px"
                      >
                        {{ $t("settings.systemBinaryNotice") }}
                      </n-alert>

                      <div class="info-list">
                        <div class="info-row">
                          <span class="info-label">{{ $t("settings.version") }}</span>
                          <n-text code>{{ ytdlpStatus?.version || "—" }}</n-text>
                        </div>
                        <div class="info-row">
                          <span class="info-label">{{ $t("settings.path") }}</span>
                          <n-ellipsis :line-clamp="1" :tooltip="{ width: 360 }">
                            {{ ytdlpStatus?.path || "—" }}
                          </n-ellipsis>
                        </div>
                      </div>

                      <n-collapse-transition :show="ytdlpDownloading">
                        <n-progress
                          type="line"
                          :percentage="Math.round(ytdlpDownloadPercent)"
                          :processing="true"
                          indicator-placement="inside"
                          :height="20"
                          :border-radius="4"
                          style="margin-top: 4px"
                        />
                      </n-collapse-transition>
                    </n-flex>
                  </n-spin>
                </n-card>
              </div>

              <div id="section-deno">
                <n-card :title="$t('settings.denoTitle')" size="small" class="section-card">
                  <template #header-extra>
                    <n-flex align="center" :size="8">
                      <n-tag v-if="!denoChecking" :type="denoStatus?.installed ? 'success' : 'error'" round>
                        {{ denoStatus?.installed ? $t("settings.installed") : $t("settings.notInstalled") }}
                      </n-tag>
                      <n-button
                        v-if="denoStatus && !denoStatus.installed"
                        :loading="denoDownloading"
                        :disabled="denoDownloading"
                        type="primary"
                        size="small"
                        strong
                        secondary
                        round
                        @click="handleDownloadDeno"
                      >
                        {{ $t("common.download") }}
                      </n-button>
                    </n-flex>
                  </template>

                  <n-spin :show="denoChecking">
                    <n-flex vertical :size="12">
                      <n-text depth="3" style="font-size: 13px">
                        {{ $t("settings.denoDesc") }}
                      </n-text>

                      <div class="info-list">
                        <div class="info-row">
                          <span class="info-label">{{ $t("settings.version") }}</span>
                          <n-text code>{{ denoStatus?.version || "—" }}</n-text>
                        </div>
                        <div class="info-row">
                          <span class="info-label">{{ $t("settings.path") }}</span>
                          <n-ellipsis :line-clamp="1" :tooltip="{ width: 360 }">
                            {{ denoStatus?.path || "—" }}
                          </n-ellipsis>
                        </div>
                      </div>

                      <n-collapse-transition :show="denoDownloading">
                        <n-progress
                          type="line"
                          :percentage="Math.round(denoDownloadPercent)"
                          :processing="true"
                          indicator-placement="inside"
                          :height="20"
                          :border-radius="4"
                          style="margin-top: 4px"
                        />
                      </n-collapse-transition>
                    </n-flex>
                  </n-spin>
                </n-card>
              </div>

              <div id="section-personal">
                <n-card :title="$t('settings.personalization')" size="small" class="section-card">
                  <div class="info-list">
                    <div class="info-row">
                      <span class="info-label">{{ $t("settings.showTaskbarProgress") }}</span>
                      <n-switch v-model:value="settingStore.showTaskbarProgress" />
                    </div>
                  </div>
                </n-card>
              </div>

              <div id="section-youtube">
                <n-card :title="$t('settings.youtubeAdvanced')" size="small" class="section-card">
                  <n-flex vertical :size="12">
                    <n-text depth="3" style="font-size: 13px">
                      {{ $t("settings.youtubeAdvancedDesc") }}
                    </n-text>
                    <div class="info-list">
                      <div class="info-row">
                        <span class="info-label">{{ $t("settings.youtubePoToken") }}</span>
                        <n-input
                          v-model:value="settingStore.youtubePoToken"
                          :placeholder="$t('settings.youtubePoTokenPlaceholder')"
                          size="small"
                          clearable
                          style="flex: 1; max-width: 480px"
                        />
                      </div>
                      <div class="info-row">
                        <span class="info-label">{{ $t("settings.youtubeVisitorData") }}</span>
                        <n-input
                          v-model:value="settingStore.youtubeVisitorData"
                          :placeholder="$t('settings.youtubeVisitorDataPlaceholder')"
                          size="small"
                          clearable
                          style="flex: 1; max-width: 480px"
                        />
                      </div>
                    </div>
                  </n-flex>
                </n-card>
              </div>

              <div id="section-download">
                <n-card :title="$t('settings.downloadOptions')" size="small" class="section-card">
                  <n-flex vertical :size="12">
                    <div class="info-list">
                      <div class="info-row">
                        <span class="info-label">{{ $t("settings.proxy") }}</span>
                        <n-input
                          v-model:value="settingStore.proxy"
                          :placeholder="$t('settings.proxyPlaceholder')"
                          size="small"
                          clearable
                          style="width: 220px"
                        />
                      </div>
                    </div>
                    <div class="info-list">
                      <div class="info-row">
                        <span class="info-label">{{ $t("settings.concurrentFragments") }}</span>
                        <n-select
                          v-model:value="settingStore.concurrentFragments"
                          :options="concurrentFragmentsOptions"
                          size="small"
                          style="width: 120px"
                        />
                      </div>
                    </div>
                    <div class="info-list">
                      <div class="info-row">
                        <span class="info-label">{{ $t("settings.maxConcurrentDownloads") }}</span>
                        <n-select
                          v-model:value="settingStore.maxConcurrentDownloads"
                          :options="maxConcurrentOptions"
                          size="small"
                          style="width: 120px"
                        />
                      </div>
                    </div>
                    <div class="info-list">
                      <div class="info-row">
                        <span class="info-label">{{ $t("settings.downloadNotification") }}</span>
                        <n-select
                          v-model:value="settingStore.notifyMode"
                          :options="notifyModeOptions"
                          size="small"
                          style="width: 120px"
                        />
                      </div>
                    </div>
                    <div class="info-list">
                      <div class="info-row">
                        <span class="info-label">{{ $t("settings.noOverwrites") }}</span>
                        <n-switch v-model:value="settingStore.noOverwrites" />
                      </div>
                    </div>
                  </n-flex>
                </n-card>
              </div>
            </n-collapse-item>
          </n-collapse>
        </div>

        <div id="section-about">
          <n-card :title="$t('settings.about')" size="small" class="section-card">
            <template #header-extra>
              <n-button
                :loading="appUpdateChecking"
                strong
                secondary
                round
                size="small"
                @click="handleCheckAppUpdate"
              >
                {{ $t("settings.checkAppUpdate") }}
              </n-button>
            </template>
            <n-flex vertical :size="8">
              <n-text depth="3" style="font-size: 13px">
                {{ $t("settings.aboutDesc") }}
              </n-text>
              <div class="info-list">
                <div class="info-row">
                  <span class="info-label">{{ $t("settings.version") }}</span>
                  <n-text code>v{{ appVersion }}</n-text>
                </div>
                <div class="info-row">
                  <span class="info-label">{{ $t("settings.platform") }}</span>
                  <n-text code>{{ platformLabel }}</n-text>
                </div>
                <div class="info-row">
                  <span class="info-label">{{ $t("settings.license") }}</span>
                  <n-text code>MIT</n-text>
                </div>
                <div class="info-row">
                  <span class="info-label">{{ $t("settings.repository") }}</span>
                  <n-button
                    text
                    tag="a"
                    href="https://github.com/MaximeSahuc/mp3-buddy"
                    target="_blank"
                    size="tiny"
                  >
                    GitHub
                  </n-button>
                </div>
              </div>
            </n-flex>
          </n-card>
        </div>

      </div>
      </n-config-provider>
    </div>
  </div>
</template>

<style scoped lang="scss">
.settings-page {
  width: 100%;
  height: calc(100vh - var(--titlebar-height));
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: linear-gradient(135deg, #1a4f9e 0%, #2d6fbd 100%);
  flex-shrink: 0;
}

.settings-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.settings-header-title {
  color: #fff;
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.3px;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
}

.settings-header-right {
  display: flex;
  align-items: center;
  gap: 2px;
}

.settings-btn {
  color: rgba(255, 255, 255, 0.9) !important;

  :deep(.n-icon) {
    color: rgba(255, 255, 255, 0.85) !important;
  }

  &:hover {
    background: rgba(255, 255, 255, 0.15) !important;
  }
}

.settings-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.settings-nav {
  width: 195px;
  min-width: 195px;
  flex-shrink: 0;
  border-right: 1px solid rgba(0, 0, 0, 0.09);
  background: #f5f6f8;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.settings-nav-title {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: #6b7280;
  padding: 8px 10px 4px;
  border-bottom: 1px solid #e4e7ec;
  flex-shrink: 0;
}

.settings-nav-items {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
  display: flex;
  flex-direction: column;
}

.settings-nav-item {
  display: block;
  width: 100%;
  padding: 7px 14px;
  font-size: 12px;
  color: #344054;
  background: transparent;
  border: none;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  transition: background 0.12s, color 0.12s;

  &:hover {
    background: #e4e7ec;
    color: #1a4f9e;
  }
}

.settings-content {
  flex: 1;
  min-width: 0;
  overflow-y: auto;
  padding: 14px;
  background: #fff;
}

.section-card {
  margin-bottom: 12px;
}

.advanced-collapse {
  margin-bottom: 12px;

  // 折叠面板内的卡片去掉外间距，最后一张不留多余空白
  :deep(.section-card) {
    margin-bottom: 12px;

    &:last-child {
      margin-bottom: 0;
    }
  }
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-row {
  display: flex;
  align-items: center;
  font-size: 13px;
  min-height: 28px;

  &::before {
    order: 1;
    content: "";
    flex: 1;
    border-bottom: 1px dashed var(--n-border-color, #e0e0e6);
    margin: 0 8px;
    min-width: 20px;
  }

  > :last-child {
    order: 2;
    flex-shrink: 0;
  }
}

.info-label {
  flex-shrink: 0;
  order: 0;
}
</style>
