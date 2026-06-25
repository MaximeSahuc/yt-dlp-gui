import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { showErrorDialog } from "@/utils/format";
import { useSettingStore } from "@/stores/setting";
import { useStatusStore } from "@/stores/status";
import type {
  VideoInfo,
  VideoFormat,
  PlaylistEntry,
  DenoStatus,
  FetchedVideoData,
} from "@/types";

type SubtitleMap = NonNullable<PlaylistEntry["subtitles"]>;

/** Aggregate subtitles from all playlist entries into a union; for each language, take the tracks from the first entry that has them */
const aggregateSubtitleMap = (
  entries: PlaylistEntry[],
  field: "subtitles" | "automatic_captions",
): SubtitleMap => {
  const merged: SubtitleMap = {};
  for (const entry of entries) {
    const map = entry[field];
    if (!map) continue;
    for (const [lang, tracks] of Object.entries(map)) {
      if (!merged[lang] && tracks?.length) merged[lang] = tracks;
    }
  }
  return merged;
};

export const useVideoStore = defineStore("video", () => {
  const fetching = ref(false);

  /** Get the currently active Cookie parameters */
  const getCookieArgs = async (): Promise<{
    cookieFile: string | null;
    cookieBrowser: string | null;
  }> => {
    const settingStore = useSettingStore();
    const { cookieMode, cookieText, cookieFile, cookieBrowser } = settingStore;
    if (cookieMode === "text" && cookieText.trim()) {
      const path = await invoke<string>("save_cookie_text", { text: cookieText });
      return { cookieFile: path, cookieBrowser: null };
    }
    if (cookieMode === "file" && cookieFile) {
      return { cookieFile, cookieBrowser: null };
    }
    if (cookieMode === "browser" && cookieBrowser) {
      return { cookieFile: null, cookieBrowser };
    }
    return { cookieFile: null, cookieBrowser: null };
  };

  /** Fetch video info; returns a structured result on success, or null on failure */
  const fetchVideoInfo = async (targetUrl: string): Promise<FetchedVideoData | null> => {
    const settingStore = useSettingStore();
    fetching.value = true;
    try {
      const { cookieFile, cookieBrowser } = await getCookieArgs();
      const info = await invoke<VideoInfo>("fetch_video_info", {
        url: targetUrl,
        cookieFile,
        cookieBrowser,
        proxy: settingStore.proxy || null,
      });

      let videoInfo: VideoInfo;
      let isPlaylist = false;
      let playlistEntries: PlaylistEntry[] = [];

      if (info._type === "playlist" && info.entries?.length) {
        isPlaylist = true;
        playlistEntries = info.entries.map((e, i) => ({
          id: e.id || String(i + 1),
          title: e.title || `第 ${i + 1} P`,
          duration: e.duration ?? null,
          url: e.url || "",
        }));
        const firstEntry = info.entries[0];
        const formats: VideoFormat[] = firstEntry?.formats || info.formats || [];
        // Playlist subtitles: yt-dlp -J does not expose subtitles at the root level for playlists,
        // they must be aggregated from each entry. For each language, use the tracks from the first entry that has them.
        videoInfo = {
          ...info,
          title: info.title || firstEntry?.title || "",
          thumbnail: info.thumbnail || firstEntry?.thumbnail || "",
          duration: info.duration || firstEntry?.duration || 0,
          formats,
          subtitles: aggregateSubtitleMap(info.entries, "subtitles"),
          automatic_captions: aggregateSubtitleMap(info.entries, "automatic_captions"),
        };
      } else {
        videoInfo = info;
      }

      const formats: VideoFormat[] = videoInfo.formats || [];
      const videoFormats = formats
        .filter((f) => f.vcodec && f.vcodec !== "none" && (!f.acodec || f.acodec === "none"))
        .sort((a, b) => (b.height || 0) - (a.height || 0));
      const audioFormats = formats
        .filter((f) => f.acodec && f.acodec !== "none" && (!f.vcodec || f.vcodec === "none"))
        .sort((a, b) => (b.abr || 0) - (a.abr || 0));

      // Show Deno setup prompt if the URL is a YouTube link and Deno is not installed
      if (/youtube\.com|youtu\.be/i.test(targetUrl)) {
        try {
          const denoStatus = await invoke<DenoStatus>("get_deno_status");
          if (!denoStatus.installed) {
            const statusStore = useStatusStore();
            statusStore.showDenoSetupModal = true;
          }
        } catch {
          // ignore
        }
      }

      return {
        url: targetUrl,
        videoInfo,
        videoFormats,
        audioFormats,
        isPlaylist,
        playlistEntries,
      };
    } catch (e: unknown) {
      const raw = e instanceof Error ? e.message : String(e) || "获取视频信息失败";
      if (/err_ytdlp_not_installed/.test(raw)) {
        const statusStore = useStatusStore();
        statusStore.showYtdlpSetupModal = true;
      } else if (/Could not copy.*cookie database/i.test(raw)) {
        showErrorDialog(raw);
      } else if (/sign in|cookies/i.test(raw)) {
        const statusStore = useStatusStore();
        statusStore.showCookieModal = true;
      } else {
        showErrorDialog(raw);
      }
      return null;
    } finally {
      fetching.value = false;
    }
  };

  return {
    fetching,
    fetchVideoInfo,
    getCookieArgs,
  };
});
