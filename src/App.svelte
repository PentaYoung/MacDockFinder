<script lang="ts">
  import DockBar from "./lib/components/DockBar.svelte";
  import MenuBar from "./lib/components/MenuBar.svelte";
  import SearchOverlay from "./lib/components/SearchOverlay.svelte";
  import NotificationPanel from "./lib/components/NotificationPanel.svelte";
  import { onMount } from "svelte";
  import { setupMenuBarListeners, searchOpen } from "./lib/stores/menubar";

  onMount(() => {
    setupMenuBarListeners();
  });
</script>

<MenuBar />
<DockBar />

{#if $searchOpen}
  <SearchOverlay />
{/if}

<NotificationPanel />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
    width: 100vw;
    height: 100vh;
  }
</style>
