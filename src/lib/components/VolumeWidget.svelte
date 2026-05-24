<script lang="ts">
  import { onMount } from "svelte";

  let volume = $state(0.5);
  let muted = $state(false);
  let showPopup = $state(false);

  onMount(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const info = await invoke<{ level: number; muted: boolean }>("get_volume");
      volume = info.level;
      muted = info.muted;
    } catch {
      volume = 0.75;
    }
  });

  function toggleMute() {
    muted = !muted;
  }
</script>

<div
  class="widget"
  onclick={() => (showPopup = !showPopup)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && (showPopup = !showPopup)}
>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    {#if muted || volume < 0.01}
      <path d="M11 5L6 9H2v6h4l5 4V5z"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/>
    {:else if volume < 0.5}
      <path d="M11 5L6 9H2v6h4l5 4V5z"/><path d="M18 8a3 3 0 010 8"/>
    {:else}
      <path d="M11 5L6 9H2v6h4l5 4V5z"/><path d="M18 8a6 6 0 010 8"/>
    {/if}
  </svg>
</div>

{#if showPopup}
  <div class="popup" onclick={(e) => e.stopPropagation()} role="presentation">
    <input
      type="range"
      min="0"
      max="100"
      value={Math.round(volume * 100)}
      oninput={(e) => {
        const v = parseInt((e.target as HTMLInputElement).value) / 100;
        volume = v;
      }}
      class="slider"
    />
    <button class="mute-btn" onclick={toggleMute}>
      {muted ? "取消静音" : "静音"}
    </button>
  </div>
{/if}

{#if showPopup}
  <div class="backdrop" onclick={() => (showPopup = false)} role="presentation"></div>
{/if}

<style>
  .widget {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.7);
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
  .popup {
    position: fixed;
    top: 32px;
    right: 120px;
    background: rgba(40, 40, 40, 0.95);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 999999;
    min-width: 120px;
  }
  .slider {
    width: 100%;
    accent-color: #3b82f6;
  }
  .mute-btn {
    background: none;
    border: 1px solid rgba(255,255,255,0.2);
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
  }
  .mute-btn:hover {
    background: rgba(255,255,255,0.1);
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 999998;
  }
</style>
