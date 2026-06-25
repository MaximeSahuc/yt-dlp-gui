import { defineStore } from "pinia";

const MAX_HISTORY = 50;

export interface HistoryItem {
  url: string;
  title: string;
  time: number;
}

export const useHistoryStore = defineStore(
  "history",
  () => {
    const items = ref<HistoryItem[]>([]);

    const urls = computed(() => items.value.map((i) => i.url));

    /** Add a successfully fetched URL (deduplicated, newest first) */
    const add = (url: string, title?: string) => {
      const trimmed = url.trim();
      if (!trimmed) return;
      const idx = items.value.findIndex((i) => i.url === trimmed);
      if (idx !== -1) items.value.splice(idx, 1);
      items.value.unshift({ url: trimmed, title: title || trimmed, time: Date.now() });
      if (items.value.length > MAX_HISTORY) items.value.length = MAX_HISTORY;
    };

    /** Remove a single record */
    const remove = (url: string) => {
      const idx = items.value.findIndex((i) => i.url === url);
      if (idx !== -1) items.value.splice(idx, 1);
    };

    /** Clear all history */
    const clear = () => {
      items.value = [];
    };

    return { items, urls, add, remove, clear };
  },
  {
    persist: true,
  },
);
