<script lang="ts">
  import { notificationPanelOpen, notifications } from "../stores/menubar";

  function clearOne(id: string) {
    notifications.update((n) => n.filter((x) => x.id !== id));
  }

  function clearAll() {
    notifications.set([]);
  }
</script>

<div class="panel" class:open={$notificationPanelOpen}>
  <div class="panel-header">
    <span class="panel-title">通知</span>
    <button class="clear-btn" onclick={clearAll}>清除全部</button>
  </div>
  <div class="panel-body">
    {#if $notifications.length === 0}
      <div class="empty">暂无通知</div>
    {:else}
      {#each $notifications as notif (notif.id)}
        <div class="notif-item">
          <div class="notif-header">
            <span class="notif-title">{notif.title}</span>
            <button class="notif-close" onclick={() => clearOne(notif.id)}>✕</button>
          </div>
          <div class="notif-body">{notif.body}</div>
        </div>
      {/each}
    {/if}
  </div>
</div>

{#if $notificationPanelOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => notificationPanelOpen.set(false)}></div>
{/if}

<style>
  .panel {
    position: fixed;
    top: 28px;
    right: 0;
    width: 320px;
    max-height: calc(100vh - 28px);
    background: rgba(35, 35, 35, 0.95);
    backdrop-filter: blur(40px);
    border-left: 1px solid rgba(255, 255, 255, 0.1);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0 0 0 12px;
    z-index: 999999;
    display: flex;
    flex-direction: column;
    transform: translateX(100%);
    transition: transform 200ms ease-out;
  }
  .panel.open {
    transform: translateX(0);
  }
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }
  .panel-title {
    font-size: 14px;
    font-weight: 600;
    color: white;
  }
  .clear-btn {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    font-size: 12px;
  }
  .clear-btn:hover { color: white; }
  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
  .empty {
    text-align: center;
    color: rgba(255, 255, 255, 0.4);
    padding: 40px 0;
    font-size: 13px;
  }
  .notif-item {
    padding: 10px 12px;
    border-radius: 8px;
    margin-bottom: 4px;
  }
  .notif-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .notif-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .notif-title {
    font-size: 13px;
    font-weight: 600;
    color: white;
  }
  .notif-close {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.4);
    cursor: pointer;
    font-size: 12px;
  }
  .notif-close:hover { color: white; }
  .notif-body {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    margin-top: 4px;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 999998;
  }
</style>
