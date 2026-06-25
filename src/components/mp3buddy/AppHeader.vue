<script setup lang="ts">
import { useI18n } from "vue-i18n";
import IconMdiHeadphones from "~icons/mdi/headphones";
import IconMdiWeb from "~icons/mdi/web";
import IconMdiCog from "~icons/mdi/cog";
import { useSettingStore } from "@/stores/setting";

const { t } = useI18n();
const router = useRouter();
const settingStore = useSettingStore();

function toggleLocale() {
  settingStore.locale = settingStore.locale === "fr-FR" ? "en-US" : "fr-FR";
}
</script>

<template>
  <div class="mp3-header">
    <div class="mp3-header-left">
      <n-icon size="20" color="#fff">
        <IconMdiHeadphones />
      </n-icon>
      <span class="mp3-header-title">{{ t("mp3buddy.title") }}</span>
    </div>
    <div class="mp3-header-right">
      <n-button quaternary size="small" @click="toggleLocale" class="mp3-btn">
        <template #icon>
          <n-icon color="rgba(255,255,255,0.8)"><IconMdiWeb /></n-icon>
        </template>
        <span class="mp3-lang">{{ settingStore.locale === "fr-FR" ? "FR" : "EN" }}</span>
      </n-button>
      <n-button quaternary size="small" @click="router.push({ name: 'settings' })" class="mp3-btn">
        <template #icon>
          <n-icon color="rgba(255,255,255,0.8)"><IconMdiCog /></n-icon>
        </template>
      </n-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.mp3-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: linear-gradient(135deg, #1a4f9e 0%, #2d6fbd 100%);
}

.mp3-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mp3-header-title {
  color: #fff;
  font-size: 16px;
  font-weight: 700;
  letter-spacing: 0.3px;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
}

.mp3-header-right {
  display: flex;
  align-items: center;
  gap: 2px;
}

.mp3-btn {
  color: rgba(255, 255, 255, 0.9) !important;

  &:hover {
    background: rgba(255, 255, 255, 0.15) !important;
  }
}

.mp3-lang {
  font-size: 11px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}
</style>
