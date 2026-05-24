<script lang="ts">
  import { notificationPanelOpen, notifications } from "../stores/menubar";
</script>

<div
  class="widget"
  onclick={() => notificationPanelOpen.update((v) => !v)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && notificationPanelOpen.update((v) => !v)}
>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/>
    <path d="M13.73 21a2 2 0 01-3.46 0"/>
  </svg>
  {#if $notifications.length > 0}
    <span class="badge">{Math.min($notifications.length, 99)}</span>
  {/if}
</div>

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
    position: relative;
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
  .badge {
    position: absolute;
    top: 1px;
    right: 1px;
    background: #ef4444;
    color: white;
    font-size: 9px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
