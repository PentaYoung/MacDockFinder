<script lang="ts">
  import { calcMagnification } from "../utils/animation";
  import { mouseX, pinnedItems } from "../stores/dock";
  import { settings } from "../stores/settings";
  import type { PinnedItem } from "../stores/dock";
  import ContextMenu from "./ContextMenu.svelte";

  let {
    item,
    isRunning = false,
    onDragStart = undefined,
  }: {
    item: PinnedItem;
    isRunning?: boolean;
    onDragStart?: (e: DragEvent) => void;
  } = $props();

  let isHovering = $state(false);
  let index = $state(0);
  let itemCount = $state(1);
  let contextMenu = $state<{ x: number; y: number } | null>(null);

  $effect(() => {
    const all = $pinnedItems;
    const found = all.findIndex((i) => i.id === item.id);
    if (found >= 0) {
      index = found;
      itemCount = all.length;
    }
  });

  const scale = $derived.by(() => {
    if (!$settings.magnification) return 1;
    return calcMagnification(index, itemCount, $mouseX, $settings.icon_size, 8);
  });

  function appColor(label: string): string {
    const colors: [string, string][] = [
      ["文件资源管理器", "#4A90D9"],
      ["终端", "#2D2D2D"],
      ["Microsoft Edge", "#1B8B4C"],
      ["Edge", "#1B8B4C"],
      ["计算器", "#3061A3"],
      ["记事本", "#B37A3A"],
      ["chrome", "#E8432A"],
      ["code", "#0078D4"],
      ["slack", "#4A154B"],
      ["spotify", "#1DB954"],
      ["notion", "#000000"],
      ["微信", "#07C160"],
      ["wechat", "#07C160"],
      ["qq", "#12B7F5"],
      ["terminal", "#2D2D2D"],
      ["explorer", "#4A90D9"],
      ["calculator", "#3061A3"],
      ["notepad", "#B37A3A"],
    ];
    const lower = label.toLowerCase();
    for (const [key, color] of colors) {
      if (lower.includes(key.toLowerCase())) return color;
    }
    return "#3b82f6";
  }

  function iconText(label: string): string {
    const specials: Record<string, string> = {
      "终端": "&gt;_",
      "计算器": "+",
    };
    return specials[label] || label[0];
  }

  function iconFont(label: string): string {
    if (label === "终端") return "monospace";
    return "system-ui,sans-serif";
  }

  const iconSrc = $derived(
    item.icon_path || `data:image/svg+xml,${encodeURIComponent(
      `<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
        <defs>
          <linearGradient id="g" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="${appColor(item.label)}"/>
            <stop offset="100%" stop-color="#000000" stop-opacity="0.35"/>
          </linearGradient>
          <filter id="s">
            <feDropShadow dx="0" dy="1" stdDeviation="2" flood-color="rgba(0,0,0,0.3)"/>
          </filter>
        </defs>
        <rect width="64" height="64" rx="14" fill="url(#g)" filter="url(#s)"/>
        <rect width="64" height="64" rx="14" fill="none" stroke="rgba(255,255,255,0.15)" stroke-width="1"/>
        <rect x="4" y="4" width="56" height="28" rx="10" fill="rgba(255,255,255,0.06)"/>
        <text x="32" y="42" font-size="26" fill="white" text-anchor="middle" font-family="${iconFont(item.label)}" font-weight="600">${iconText(item.label)}</text>
      </svg>`
    )}`
  );

  async function handleClick() {
    if (!item.path) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("launch_app", { path: item.path });
    } catch {
      // outside Tauri
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleClick();
    }
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  async function removeFromDock() {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("remove_pinned_item", { id: item.id });
      pinnedItems.update((items) => items.filter((i) => i.id !== item.id));
    } catch {
      pinnedItems.update((items) => items.filter((i) => i.id !== item.id));
    }
  }
</script>

<div
  class="dock-item"
  class:running={isRunning}
  class:hovering={isHovering}
  role="button"
  tabindex="0"
  aria-label={item.label}
  onclick={handleClick}
  onkeydown={handleKeydown}
  oncontextmenu={handleContextMenu}
  onmouseenter={() => (isHovering = true)}
  onmouseleave={() => (isHovering = false)}
  style="transform: scale({scale});"
  draggable="true"
  ondragstart={onDragStart}
>
  <div class="icon-wrapper" style="width: {$settings.icon_size}px; height: {$settings.icon_size}px;">
    <img src={iconSrc} alt={item.label} class="icon" draggable="false" />
  </div>
  {#if isHovering}
    <div class="tooltip">{item.label}</div>
  {/if}
  {#if isRunning}
    <div class="running-dot"></div>
  {/if}
</div>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    items={[
      { label: "打开", action: handleClick },
      { label: "从Dock移除", action: removeFromDock },
    ]}
    onclose={closeContextMenu}
  />
{/if}

<style>
  .dock-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    cursor: pointer;
    transition: transform 100ms ease-out;
    position: relative;
    user-select: none;
  }

  .icon-wrapper {
    border-radius: 12px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.1);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    transition: border-radius 150ms ease-out, box-shadow 150ms ease-out, transform 150ms ease-out;
  }

  .dock-item.hovering .icon-wrapper {
    border-radius: 14px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5), 0 0 30px rgba(255, 255, 255, 0.12);
    transform: translateY(-4px);
  }

  .icon {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .tooltip {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    background: rgba(0, 0, 0, 0.85);
    color: white;
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 12px;
    white-space: nowrap;
    pointer-events: none;
    backdrop-filter: blur(10px);
  }

  .running-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.7);
    margin-top: 3px;
    transition: background 150ms;
  }

  .dock-item.hovering .running-dot {
    background: rgba(255, 255, 255, 0.9);
  }
</style>
