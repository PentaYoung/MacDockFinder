<script lang="ts">
  import { onMount } from "svelte";
  import DockItem from "./DockItem.svelte";
  import DockDivider from "./DockDivider.svelte";
  import TrashBin from "./TrashBin.svelte";
  import {
    pinnedItems, activeWindows, runningAppNames,
    loadPinnedItems, setupWindowListeners,
    mouseX, mouseY, dockVisible,
    type PinnedItem
  } from "../stores/dock";
  import { settings, loadSettings } from "../stores/settings";

  let isVisible = $state(true);
  let hideTimeout: ReturnType<typeof setTimeout> | null = null;
  let draggedItemId: string | null = $state(null);

  onMount(async () => {
    await loadSettings();
    await loadPinnedItems();
    await setupWindowListeners();

    document.addEventListener("mousemove", handleMouseMove);
  });

  function handleMouseMove(e: MouseEvent) {
    mouseX.set(e.clientX);
    mouseY.set(e.clientY);

    if ($settings.auto_hide) {
      const barHeight = 100;
      if (e.clientY >= window.innerHeight - 5) {
        if (hideTimeout) clearTimeout(hideTimeout);
        isVisible = true;
      } else if (e.clientY < window.innerHeight - barHeight - 20) {
        if (hideTimeout) clearTimeout(hideTimeout);
        hideTimeout = setTimeout(() => {
          isVisible = false;
        }, 300);
      }
    }
  }

  function handleItemDragStart(e: DragEvent, id: string) {
    draggedItemId = id;
    e.dataTransfer?.setData("text/plain", id);
  }

  function handleDragOver(e: DragEvent, targetId: string) {
    if (!draggedItemId || draggedItemId === targetId) return;
    e.preventDefault();
    pinnedItems.update((items) => {
      const from = items.findIndex((i) => i.id === draggedItemId);
      const to = items.findIndex((i) => i.id === targetId);
      if (from < 0 || to < 0) return items;
      const newItems = [...items];
      const [moved] = newItems.splice(from, 1);
      newItems.splice(to, 0, moved);
      return newItems;
    });
  }

  async function handleDrop() {
    if (!draggedItemId) return;
    const ids = $pinnedItems.map((i) => i.id);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("reorder_items", { ids });
    } catch {
      // outside Tauri
    }
    draggedItemId = null;
  }
</script>

<div
  class="dock-bar"
  class:visible={isVisible && $dockVisible}
  class:hidden={!isVisible || !$dockVisible}
  style="--icon-size: {$settings.icon_size}px;"
>
  <div class="dock-inner">
    <div class="dock-items">
      {#each $pinnedItems as item (item.id)}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="dock-item-slot"
          ondragover={(e) => handleDragOver(e, item.id)}
          ondrop={handleDrop}
          ondragenter={(e) => e.preventDefault()}
          dragover={(e) => e.preventDefault()}
        >
          <DockItem {item} onDragStart={(e) => handleItemDragStart(e, item.id)} />
        </div>
      {/each}
      {#if $activeWindows.length > 0}
        <DockDivider />
        {#each $activeWindows as win (win.hwnd)}
          <DockItem
            item={{ id: `win-${win.hwnd}`, path: "", label: win.app_name, icon_path: win.icon_base64 }}
            isRunning={true}
          />
        {/each}
      {/if}
      <DockDivider />
      <TrashBin />
    </div>
  </div>
</div>

<style>
  .dock-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    justify-content: center;
    padding: 8px 0;
    transition: transform 200ms ease-in, opacity 200ms ease-in;
    z-index: 999999;
    pointer-events: auto;
  }

  .dock-bar.hidden {
    transform: translateY(120%);
    opacity: 0;
    pointer-events: none;
  }

  .dock-bar.visible {
    transform: translateY(0);
    opacity: 1;
  }

  .dock-inner {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px 12px;
    background: rgba(30, 30, 30, 0.65);
    backdrop-filter: blur(40px) saturate(180%);
    -webkit-backdrop-filter: blur(40px) saturate(180%);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.4),
      inset 0 0 0 1px rgba(255, 255, 255, 0.05);
  }

  .dock-items {
    display: flex;
    align-items: flex-end;
    gap: 4px;
  }

  .dock-item-slot {
    display: flex;
  }
</style>
