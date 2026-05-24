<script lang="ts">
  import { onMount } from "svelte";

  let connected = $state(false);
  let signal = $state(0);
  let ssid = $state("");

  onMount(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const info = await invoke<{ ssid: string; signal: number; connected: boolean }>("get_wifi");
      connected = info.connected;
      signal = info.signal;
      ssid = info.ssid;
    } catch {
      connected = false;
    }
  });
</script>

<div class="widget" title={connected ? `${ssid} (信号: ${signal}/4)` : "未连接"}>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    {#if connected}
      <path d="M12 18h.01" stroke-width="2" stroke-linecap="round"/>
      {#if signal >= 2}
        <path d="M5.5 13a8 8 0 0113 0" opacity="0.6"/>
      {/if}
      {#if signal >= 3}
        <path d="M2 9.5a12 12 0 0120 0" opacity="0.3"/>
      {/if}
      {#if signal >= 4}
        <path d="M-1 6a16 16 0 0126 0" opacity="0.15"/>
      {/if}
    {:else}
      <path d="M12 18h.01"/>
      <path d="M5.5 13a8 8 0 0113 0"/>
      <path d="M2 9.5a12 12 0 0120 0"/>
      <line x1="2" y1="2" x2="22" y2="22" stroke="red"/>
    {/if}
  </svg>
</div>

<style>
  .widget {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: default;
    color: rgba(255, 255, 255, 0.7);
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
</style>
