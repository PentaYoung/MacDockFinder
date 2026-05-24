<script lang="ts">
  let {
    x,
    y,
    items,
    onclose,
  }: {
    x: number;
    y: number;
    items: { label: string; action: () => void }[];
    onclose: () => void;
  } = $props();

  function handleItemClick(action: () => void) {
    action();
    onclose();
  }

  function handleBackdropClick() {
    onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="backdrop"
  onclick={handleBackdropClick}
  onkeydown={(e) => e.key === "Escape" && onclose()}
  oncontextmenu={(e) => e.preventDefault()}
  role="button"
  tabindex="-1"
>
  <div
    class="menu"
    style="left: {x}px; top: {y}px;"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onclose()}
    role="menu"
    tabindex="-1"
  >
    {#each items as item}
      <div
        class="menu-item"
        onclick={() => handleItemClick(item.action)}
        onkeydown={(e) => (e.key === "Enter" || e.key === " ") && handleItemClick(item.action)}
        role="menuitem"
        tabindex="0"
      >
        {item.label}
      </div>
    {/each}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999999;
  }

  .menu {
    position: fixed;
    background: rgba(40, 40, 45, 0.95);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 4px 0;
    min-width: 140px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    z-index: 99999999;
  }

  .menu-item {
    padding: 6px 16px;
    font-size: 13px;
    color: rgba(255, 255, 255, 0.8);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
  }

  .menu-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
