<script lang="ts">
  let { app = null }: { app: { name: string; icon?: string } | null } = $props();
</script>

<div class="app-indicator">
  {#if app}
    <span class="app-name">{app.name}</span>
  {:else}
    <span class="app-name">Finder</span>
  {/if}
</div>

<style>
  .app-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    color: white;
    padding: 0 6px;
    height: 22px;
    line-height: 22px;
  }
  .app-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }
</style>
