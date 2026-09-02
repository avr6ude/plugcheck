<script lang="ts">
  import { onMount } from "svelte";
  import { store, startPolling, refresh, verdictFor } from "$lib/snapshot.svelte";
  import PortCard from "$lib/PortCard.svelte";
  import EngineerPanel from "$lib/EngineerPanel.svelte";

  let engineerPort = $state<string | null>(null);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    startPolling().then((u) => (unlisten = u));
    return () => unlisten?.();
  });
</script>

<main>
  <header>
    <h1>plugcheck</h1>
    <button onclick={refresh} disabled={store.loading}>
      {store.loading ? "…" : "Refresh"}
    </button>
  </header>

  {#if store.error}
    <p class="err">{store.error}</p>
  {:else if store.snapshot}
    <div class="cards">
      {#each store.snapshot.ports as port (port.id)}
        <PortCard
          {port}
          verdict={verdictFor(port.id)}
          onEngineer={(id) => (engineerPort = id)}
        />
      {/each}
    </div>
  {:else}
    <p class="muted">Reading ports…</p>
  {/if}

  {#if engineerPort}
    <EngineerPanel portId={engineerPort} onClose={() => (engineerPort = null)} />
  {/if}
</main>

<style>
  :global(:root) {
    --bg: #fbfbfa;
    --fg: #1a1a1a;
    --muted: #6b6b6b;
    --card: #ffffff;
    --line: #e2e2e0;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #1c1c1e;
      --fg: #f2f2f2;
      --muted: #9a9a9a;
      --card: #262628;
      --line: #3a3a3c;
    }
  }
  :global(body) {
    margin: 0;
    background: var(--bg);
    color: var(--fg);
    font: 14px/1.5 -apple-system, system-ui, sans-serif;
  }
  main {
    max-width: 520px;
    margin: 0 auto;
    padding: 1rem;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
  }
  h1 {
    margin: 0;
    font-size: 1.3rem;
  }
  header button {
    font-size: 0.8rem;
    padding: 0.35rem 0.8rem;
    border: 1px solid var(--line);
    border-radius: 7px;
    background: var(--card);
    color: var(--fg);
    cursor: pointer;
  }
  header button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .cards {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .err {
    color: #cf222e;
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
  }
  .muted {
    color: var(--muted);
  }
</style>
