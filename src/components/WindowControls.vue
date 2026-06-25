<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

withDefaults(defineProps<{ dark?: boolean }>(), { dark: true });

const win = getCurrentWindow();
</script>

<template>
  <div class="win-controls" :class="{ 'win-controls--light': !dark }">
    <button class="wc-btn wc-minimize" @click.stop="win.minimize()" :title="$t('window.minimize')">
      <span class="wc-icon">&#x2013;</span>
    </button>
    <button class="wc-btn wc-maximize" @click.stop="win.toggleMaximize()" :title="$t('window.maximize')">
      <span class="wc-icon">&#x25A1;</span>
    </button>
    <button class="wc-btn wc-close" @click.stop="win.close()" :title="$t('window.close')">
      <span class="wc-icon">&#x2715;</span>
    </button>
  </div>
</template>

<style scoped lang="scss">
.win-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;

  .wc-btn {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    color: rgba(255, 255, 255, 0.8);
    transition: background 0.12s, color 0.12s;
    padding: 0;
    font-size: 13px;
    line-height: 1;

    &:hover {
      background: rgba(255, 255, 255, 0.18);
      color: #fff;
    }

    &.wc-close:hover {
      background: #e83535;
      color: #fff;
    }
  }

  &.win-controls--light .wc-btn {
    color: rgba(0, 0, 0, 0.45);

    &:hover {
      background: rgba(0, 0, 0, 0.08);
      color: rgba(0, 0, 0, 0.75);
    }

    &.wc-close:hover {
      background: #e83535;
      color: #fff;
    }
  }

  .wc-icon {
    pointer-events: none;
    user-select: none;
    line-height: 1;
  }
}
</style>
