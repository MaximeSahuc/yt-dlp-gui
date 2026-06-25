import { createRouter, createWebHistory } from "vue-router";
// Home and Settings pages are statically imported: they are frequently switched between, and lazy loading would cause a white-screen flash on first open
import Mp3Buddy from "@/pages/Mp3Buddy.vue";
import Settings from "@/pages/Settings.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: Mp3Buddy,
    },
    {
      path: "/downloads",
      name: "downloads",
      component: () => import("@/pages/Downloads.vue"),
    },
    {
      path: "/toolbox",
      component: () => import("@/pages/Toolbox.vue"),
      children: [
        {
          path: "",
          name: "toolbox",
          component: () => import("@/pages/toolbox/ToolList.vue"),
        },
        {
          path: "thumbnail",
          name: "toolbox-thumbnail",
          component: () => import("@/pages/toolbox/Thumbnail.vue"),
        },
        {
          path: "subtitles",
          name: "toolbox-subtitles",
          component: () => import("@/pages/toolbox/Subtitles.vue"),
        },
        {
          path: "chapters",
          name: "toolbox-chapters",
          component: () => import("@/pages/toolbox/Chapters.vue"),
        },
        {
          path: "comments",
          name: "toolbox-comments",
          component: () => import("@/pages/toolbox/Comments.vue"),
        },
        {
          path: "plugins",
          name: "toolbox-plugins",
          component: () => import("@/pages/toolbox/Plugins.vue"),
        },
      ],
    },
    {
      path: "/settings",
      name: "settings",
      component: Settings,
    },
  ],
});

export default router;
