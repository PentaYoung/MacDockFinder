<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  let now = $state(new Date());

  let timer: ReturnType<typeof setInterval>;

  onMount(() => {
    timer = setInterval(() => {
      now = new Date();
    }, 1000);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  let display = $derived(
    now.toLocaleDateString("zh-CN", {
      month: "numeric",
      day: "numeric",
      weekday: "short",
      hour: "2-digit",
      minute: "2-digit",
    })
  );
</script>

<div class="datetime">
  {display}
</div>

<style>
  .datetime {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.85);
    padding: 0 8px;
    height: 22px;
    line-height: 22px;
    border-radius: 4px;
    cursor: default;
  }
  .datetime:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
