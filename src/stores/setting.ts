import { defineStore } from "pinia";
import { setI18nLocale, resolveLocale } from "@/locales";

export const useSettingStore = defineStore(
  "setting",
  () => {
    /** UI language */
    const locale = ref(resolveLocale(""));

    watch(locale, (val) => {
      setI18nLocale(val);
    });

    /** Theme mode */
    const themeMode = ref<"auto" | "light" | "dark">("light");

    /** Download directory */
    const downloadDir = ref("");

    /** Cookie mode */
    const cookieMode = ref<"none" | "text" | "file" | "browser">("none");

    /** Cookie text content (Netscape format) */
    const cookieText = ref("");

    /** Cookie file path */
    const cookieFile = ref("");

    /** Browser name for reading cookies from browser */
    const cookieBrowser = ref("chrome");

    /** Proxy address */
    const proxy = ref("");

    /** Filename output template */
    const outputTemplate = ref("%(title).200s [%(id)s].%(ext)s");

    /** Concurrent fragment count, 0 = disabled */
    const concurrentFragments = ref(0);

    /** Do not overwrite existing files */
    const noOverwrites = ref(false);

    /** Maximum concurrent downloads, 0 = unlimited */
    const maxConcurrentDownloads = ref(0);

    /** Download completion notification mode */
    const notifyMode = ref<"none" | "app" | "system" | "all">("app");

    /** Automatically check for updates on startup */
    const autoCheckUpdate = ref(true);

    /** YouTube PO Token (used to bypass 403 / rate limiting) */
    const youtubePoToken = ref("");

    /** YouTube visitor_data (paired with PO Token) */
    const youtubeVisitorData = ref("");

    /** Show download progress in the taskbar */
    const showTaskbarProgress = ref(true);

    return {
      locale,
      themeMode,
      downloadDir,
      cookieMode,
      cookieText,
      cookieFile,
      cookieBrowser,
      proxy,
      outputTemplate,
      concurrentFragments,
      noOverwrites,
      maxConcurrentDownloads,
      notifyMode,
      autoCheckUpdate,
      youtubePoToken,
      youtubeVisitorData,
      showTaskbarProgress,
    };
  },
  {
    persist: true,
  },
);
