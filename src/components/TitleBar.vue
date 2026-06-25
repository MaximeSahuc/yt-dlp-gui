<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useOsTheme } from "naive-ui";
import { useSettingStore } from "@/stores/setting";

const win = getCurrentWindow();
const settingStore = useSettingStore();
const osTheme = useOsTheme();

const isDark = computed(
  () =>
    settingStore.themeMode === "dark" ||
    (settingStore.themeMode === "auto" && osTheme.value === "dark"),
);
</script>

<template>
  <div class="titlebar" :class="{ dark: isDark }" data-tauri-drag-region>
    <span data-tauri-drag-region style="flex: 1" />
<div class="titlebar-controls">
      <button class="tb-btn tb-minimize" @click.stop="win.minimize()" :title="$t('window.minimize')" />
      <button class="tb-btn tb-maximize" @click.stop="win.toggleMaximize()" :title="$t('window.maximize')" />
      <button class="tb-btn tb-close" @click.stop="win.close()" :title="$t('window.close')" />
    </div>
  </div>
</template>

<style scoped lang="scss">
.titlebar {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  background: #dcdcdc;
  flex-shrink: 0;

  &.dark {
    background: #252525;
  }
}

.titlebar-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tb-btn {
  width: 13px;
  height: 13px;
  border-radius: 50%;
  border: none;
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: filter 0.1s;
  flex-shrink: 0;

  &:hover {
    filter: brightness(0.85);
  }

  &.tb-close {
    background: #ff5f57;
  }

  &.tb-minimize {
    background: #febc2e;
  }

  &.tb-maximize {
    background: #28c840;
  }
}
</style>
