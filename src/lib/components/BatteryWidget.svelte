<script lang="ts">
  import { onMount } from "svelte";

  let percent = $state(100);
  let charging = $state(true);

  onMount(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const info = await invoke<{ percent: number; charging: boolean }>("get_battery");
      percent = info.percent;
      charging = info.charging;
    } catch {
      percent = 85;
      charging = false;
    }
  });

  let color = $derived(
    percent > 50 ? "#4ade80" : percent > 20 ? "#fbbf24" : "#ef4444"
  );
</script>

<div class="widget" title={`${percent}%${charging ? " 充电中" : ""}`}>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    <rect x="2" y="7" width="18" height="10" rx="2" fill="none"/>
    <rect x="2" y="7" width={18 * percent / 100} height="10" rx="2" fill={color} opacity="0.8"/>
    <path d="M22 11v2" stroke-width="2"/>
    {#if charging}
      <path d="M8 12l3-3v2h3l-3 3v-2H8z" fill="white" stroke="none"/>
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
