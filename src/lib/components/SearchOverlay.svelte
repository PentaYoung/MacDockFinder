<script lang="ts">
  import { searchOpen } from "../stores/menubar";

  let query = $state("");
  let results = $state<{ name: string; path: string }[]>([]);
  let selectedIndex = $state(0);

  const mockApps = [
    { name: "文件资源管理器", path: "explorer.exe" },
    { name: "Microsoft Edge", path: "msedge.exe" },
    { name: "Code", path: "code.exe" },
    { name: "终端", path: "wt.exe" },
    { name: "计算器", path: "calculator.exe" },
    { name: "记事本", path: "notepad.exe" },
  ];

  function doSearch(q: string) {
    query = q;
    if (!q.trim()) {
      results = [];
      return;
    }
    const lower = q.toLowerCase();
    results = mockApps.filter((r) =>
      r.name.toLowerCase().includes(lower)
    );
    selectedIndex = 0;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter" && results[selectedIndex]) {
      launchApp(results[selectedIndex].path);
    } else if (e.key === "Escape") {
      searchOpen.set(false);
    }
  }

  async function launchApp(path: string) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("launch_app", { path });
    } catch { /* outside Tauri */ }
    searchOpen.set(false);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="overlay" role="presentation" onclick={() => searchOpen.set(false)}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="search-panel" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="search-input-wrap">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/>
      </svg>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        type="text"
        class="search-input"
        placeholder="搜索应用..."
        autofocus
        bind:value={query}
        oninput={(e) => doSearch((e.target as HTMLInputElement).value)}
        onkeydown={handleKeydown}
      />
    </div>
    {#if results.length > 0}
      <div class="results">
        {#each results as result, i}
          <div
            class="result-item"
            class:selected={i === selectedIndex}
            onclick={() => launchApp(result.path)}
            onmouseenter={() => (selectedIndex = i)}
            role="option"
            tabindex="-1"
            aria-selected={i === selectedIndex}
          >
            <span class="result-name">{result.name}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    display: flex;
    justify-content: center;
    padding-top: 120px;
    z-index: 1000000;
  }
  .search-panel {
    width: 480px;
    max-width: 90vw;
  }
  .search-input-wrap {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    padding: 12px 16px;
    color: rgba(255, 255, 255, 0.5);
  }
  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: white;
    font-size: 18px;
  }
  .results {
    margin-top: 8px;
    background: rgba(40, 40, 40, 0.9);
    border-radius: 10px;
    overflow: hidden;
  }
  .result-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.8);
  }
  .result-item.selected {
    background: rgba(59, 130, 246, 0.3);
    color: white;
  }
  .result-name {
    font-size: 14px;
  }
</style>
