import { defineStore } from "pinia";

export const useStatusStore = defineStore("status", () => {
  /** Cookie settings modal */
  const showCookieModal = ref(false);

  /** App update modal */
  const showUpdateModal = ref(false);
  const updateVersion = ref("");
  const updateNotes = ref("");

  /** yt-dlp not installed modal */
  const showYtdlpSetupModal = ref(false);

  /** Deno not installed notification modal */
  const showDenoSetupModal = ref(false);

  return {
    showCookieModal,
    showUpdateModal,
    updateVersion,
    updateNotes,
    showYtdlpSetupModal,
    showDenoSetupModal,
  };
});
