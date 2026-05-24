<script lang="ts">
  import { searchOpen } from "../stores/menubar";
</script>

<div
  class="widget"
  onclick={() => searchOpen.set(true)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && searchOpen.set(true)}
>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    <circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/>
  </svg>
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
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
</style>
