import { createI18n } from "vue-i18n";
import zhCN from "./zh-CN.json";
import enUS from "./en-US.json";
import jaJP from "./ja-JP.json";
import koKR from "./ko-KR.json";
import esES from "./es-ES.json";
import ruRU from "./ru-RU.json";
import zhTW from "./zh-TW.json";
import arEG from "./ar-EG.json";
import deDE from "./de-DE.json";
import frFR from "./fr-FR.json";
import ptBR from "./pt-BR.json";
import viVN from "./vi-VN.json";

// ==================== Locale registry (add a new language here + create its translation file) ====================

export interface LocaleEntry {
  /** BCP 47 language code */
  code: string;
  /** Flag emoji */
  flag: string;
  /** Native display name */
  label: string;
  /** Matching rule against navigator.language prefix */
  match: (lang: string) => boolean;
  /** Whether the language is written right-to-left */
  rtl?: boolean;
}

// Ordered by ISO 639-1 code (alphabetical); Chinese variants are differentiated by region code
export const localeEntries: LocaleEntry[] = [
  {
    code: "ar-EG",
    flag: "🇪🇬",
    label: "العربية",
    match: (lang) => lang.startsWith("ar"),
    rtl: true,
  },
  { code: "de-DE", flag: "🇩🇪", label: "Deutsch", match: (lang) => lang.startsWith("de") },
  { code: "en-US", flag: "🇺🇸", label: "English", match: (lang) => lang.startsWith("en") },
  { code: "es-ES", flag: "🇪🇸", label: "Español", match: (lang) => lang.startsWith("es") },
  { code: "fr-FR", flag: "🇫🇷", label: "Français", match: (lang) => lang.startsWith("fr") },
  { code: "ja-JP", flag: "🇯🇵", label: "日本語", match: (lang) => lang.startsWith("ja") },
  { code: "ko-KR", flag: "🇰🇷", label: "한국어", match: (lang) => lang.startsWith("ko") },
  { code: "pt-BR", flag: "🇧🇷", label: "Português", match: (lang) => lang.startsWith("pt") },
  { code: "ru-RU", flag: "🇷🇺", label: "Русский", match: (lang) => lang.startsWith("ru") },
  { code: "vi-VN", flag: "🇻🇳", label: "Tiếng Việt", match: (lang) => lang.startsWith("vi") },
  {
    code: "zh-CN",
    flag: "🇨🇳",
    label: "简体中文",
    match: (lang) => lang === "zh-CN" || lang === "zh-SG" || lang === "zh",
  },
  { code: "zh-TW", flag: "🇭🇰", label: "繁體中文", match: (lang) => lang.startsWith("zh") },
];

/** Fast locale code → entry lookup */
const localeMap = new Map(localeEntries.map((e) => [e.code, e]));

// ==================== Utilities ====================

/** Return the best-matching locale code for the current system language; falls back to en-US */
const getSystemLocale = (): string => {
  const lang = navigator.language;
  const matched = localeEntries.find((e) => e.match(lang));
  return matched ? matched.code : "en-US";
};

/** Read the user's locale preference from localStorage */
const getSavedLocale = (): string | null => {
  try {
    const setting = localStorage.getItem("setting");
    if (setting) {
      const parsed = JSON.parse(setting);
      return parsed.locale || null;
    }
  } catch {
    // ignore
  }
  return null;
};

/** Resolve a locale value to a valid locale code */
export const resolveLocale = (locale: string): string => {
  if (!locale) return getSystemLocale();
  return localeMap.has(locale) ? locale : getSystemLocale();
};

// ==================== i18n instance ====================

const savedLocale = getSavedLocale();
const defaultLocale = resolveLocale(savedLocale ?? "auto");

const i18n = createI18n({
  legacy: false,
  locale: defaultLocale,
  fallbackLocale: "en-US",
  messages: {
    "ar-EG": arEG,
    "de-DE": deDE,
    "en-US": enUS,
    "es-ES": esES,
    "fr-FR": frFR,
    "ja-JP": jaJP,
    "ko-KR": koKR,
    "pt-BR": ptBR,
    "ru-RU": ruRU,
    "vi-VN": viVN,
    "zh-CN": zhCN,
    "zh-TW": zhTW,
  },
});

/** Return the document writing direction for a locale code */
const getDirection = (code: string): "rtl" | "ltr" => (localeMap.get(code)?.rtl ? "rtl" : "ltr");

/** Switch the active locale (called by the settings store) */
export const setI18nLocale = (locale: string) => {
  const resolved = resolveLocale(locale);
  (i18n.global.locale as unknown as { value: string }).value = resolved;
  document.documentElement.lang = resolved;
  document.documentElement.dir = getDirection(resolved);
};

// Sync html lang and dir on initialisation
document.documentElement.lang = defaultLocale;
document.documentElement.dir = getDirection(defaultLocale);

export default i18n;
