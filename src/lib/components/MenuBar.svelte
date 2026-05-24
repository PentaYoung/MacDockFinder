<script lang="ts">
  import AppIndicator from "./AppIndicator.svelte";
  import DateTimeWidget from "./DateTimeWidget.svelte";
  import VolumeWidget from "./VolumeWidget.svelte";
  import BatteryWidget from "./BatteryWidget.svelte";
  import WifiWidget from "./WifiWidget.svelte";
  import SearchButton from "./SearchButton.svelte";
  import NotificationBell from "./NotificationBell.svelte";
  import { activeApp, menuBarVisible } from "../stores/menubar";
</script>

<div class="menu-bar" class:hidden={!$menuBarVisible}>
  <div class="menu-left">
    <AppIndicator app={$activeApp} />
  </div>
  <div class="menu-right">
    <WifiWidget />
    <VolumeWidget />
    <BatteryWidget />
    <SearchButton />
    <NotificationBell />
    <DateTimeWidget />
  </div>
</div>

<style>
  .menu-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background: rgba(30, 30, 30, 0.7);
    backdrop-filter: blur(40px) saturate(180%);
    -webkit-backdrop-filter: blur(40px) saturate(180%);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    z-index: 999998;
    user-select: none;
    transition: transform 200ms ease-in, opacity 200ms ease-in;
  }
  .menu-bar.hidden {
    transform: translateY(-100%);
    opacity: 0;
    pointer-events: none;
  }
  .menu-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 200px;
  }
  .menu-right {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
  }
</style>
