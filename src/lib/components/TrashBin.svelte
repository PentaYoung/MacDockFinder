<script lang="ts">
  import { pinnedItems } from "../stores/dock";

  let isDraggingOver = $state(false);

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDraggingOver = true;
  }

  function handleDragLeave() {
    isDraggingOver = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDraggingOver = false;

    const id = e.dataTransfer?.getData("text/plain");
    if (!id) return;

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("remove_pinned_item", { id });
      pinnedItems.update((items) => items.filter((i) => i.id !== id));
    } catch {
      pinnedItems.update((items) => items.filter((i) => i.id !== id));
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
    }
  }
</script>

<div
  class="trash-bin"
  class:dragover={isDraggingOver}
  role="button"
  tabindex="0"
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  onkeydown={handleKeydown}
  aria-label="拖拽图标至此移除"
>
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    <path d="M3 6h18M8 6V4a1 1 0 011-1h6a1 1 0 011 1v2M19 6l-1.5 14a2 2 0 01-2 1.5H8.5a2 2 0 01-2-1.5L5 6"/>
  </svg>
</div>

<style>
  .trash-bin {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-size);
    height: var(--icon-size);
    color: rgba(255, 255, 255, 0.4);
    transition: color 150ms ease-out, transform 150ms ease-out;
    cursor: default;
    flex-shrink: 0;
  }

  .trash-bin.dragover {
    color: rgba(255, 80, 80, 0.9);
    transform: scale(1.2);
  }
</style>
