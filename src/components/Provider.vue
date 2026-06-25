<!-- Global config provider -->
<template>
  <n-config-provider
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
    :theme="theme"
    :theme-overrides="themeOverrides"
    abstract
    inline-theme-disabled
    preflight-style-disabled
  >
    <n-global-style />
    <n-loading-bar-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-message-provider :max="1" placement="bottom">
            <n-modal-provider>
              <slot />
              <NaiveProviderContent />
            </n-modal-provider>
          </n-message-provider>
        </n-notification-provider>
      </n-dialog-provider>
    </n-loading-bar-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import {
  zhCN,
  dateZhCN,
  enUS,
  dateEnUS,
  darkTheme,
  useOsTheme,
  useLoadingBar,
  useModal,
  useDialog,
  useMessage,
  useNotification,
  GlobalThemeOverrides,
} from "naive-ui";
import { useI18n } from "vue-i18n";
import { useSettingStore } from "@/stores/setting";

const settingStore = useSettingStore();
const { locale } = useI18n();

// Naive UI locale: Chinese locales use zhCN, everything else uses enUS
const naiveLocale = computed(() => (locale.value.startsWith("zh") ? zhCN : enUS));
const naiveDateLocale = computed(() => (locale.value.startsWith("zh") ? dateZhCN : dateEnUS));

const osTheme = useOsTheme();

const themeOverrides = shallowRef<GlobalThemeOverrides>({
  common: {
    borderRadius: "8px",
  },
});

const theme = computed(() => {
  return settingStore.themeMode === "auto"
    ? // follow system
      osTheme.value === "dark"
      ? darkTheme
      : null
    : // custom
      settingStore.themeMode === "dark"
      ? darkTheme
      : null;
});

// Sync the dark class to <html> so scoped CSS variables can toggle the palette
watchEffect(() => {
  document.documentElement.classList.toggle("dark", theme.value === darkTheme);
});

// Mount Naive UI utility providers
const NaiveProviderContent = defineComponent({
  setup() {
    window.$loadingBar = useLoadingBar();
    window.$notification = useNotification();
    window.$message = useMessage();
    window.$dialog = useDialog();
    window.$modal = useModal();
  },
  render() {
    return h("div", { className: "main-tools" });
  },
});
</script>
