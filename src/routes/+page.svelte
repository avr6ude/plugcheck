<script lang="ts">
  import { onMount } from "svelte";
  import { store, startPolling, refresh, verdictFor } from "$lib/snapshot.svelte";
  import PortCard from "$lib/PortCard.svelte";
  import EngineerPanel from "$lib/EngineerPanel.svelte";

  let engineerPort = $state<string | null>(null);
  let hideEmpty = $state(load("plugcheck.hideEmpty", false));

  $effect(() => save("plugcheck.hideEmpty", hideEmpty));

  onMount(() => {
    let unlisten: (() => void) | undefined;
    startPolling().then((u) => (unlisten = u));
    return () => unlisten?.();
  });

  const ports = $derived(store.snapshot?.ports ?? []);
  const shown = $derived(hideEmpty ? ports.filter((p) => p.occupied) : ports);
  const emptyCount = $derived(ports.filter((p) => !p.occupied).length);

  function load(k: string, d: boolean): boolean {
    try {
      const v = localStorage.getItem(k);
      return v == null ? d : v === "1";
    } catch {
      return d;
    }
  }
  function save(k: string, v: boolean) {
    try {
      localStorage.setItem(k, v ? "1" : "0");
    } catch {
      /* private window / blocked */
    }
  }
</script>

<main>
  <header>
    <div class="brand">
      <h1>plugcheck</h1>
      <span>USB-C &amp; Thunderbolt inspector</span>
    </div>
    <button class="refresh" onclick={refresh} disabled={store.loading} title="Refresh now">
      {store.loading ? "…" : "↻"}
    </button>
  </header>

  {#if emptyCount > 0}
    <label class="toggle">
      <input type="checkbox" bind:checked={hideEmpty} />
      Hide {emptyCount} empty port{emptyCount === 1 ? "" : "s"}
    </label>
  {/if}

  {#if store.error}
    <p class="err">{store.error}</p>
  {:else if store.snapshot}
    <div class="list">
      {#each shown as port (port.id)}
        {#if port.occupied}
          <PortCard
            {port}
            verdict={verdictFor(port.id)}
            onEngineer={(id) => (engineerPort = id)}
          />
        {:else}
          <button class="empty-row" onclick={() => (engineerPort = port.id)}>
            <span class="pip"></span>
            <span class="lbl">{port.kind}</span>
            <span class="tag">empty</span>
          </button>
        {/if}
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
    --line: #e7e7e4;
    --ok: #2ea043;
    --warn: #bf8700;
    --warn-fg: #8a6300;
    --bad: #cf222e;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #1b1b1d;
      --fg: #f0f0f0;
      --muted: #9a9a9a;
      --card: #262629;
      --line: #38383b;
      --ok: #3fb950;
      --warn: #d29922;
      --warn-fg: #e3b341;
      --bad: #f85149;
    }
  }
  :global(body) {
    margin: 0;
    background: var(--bg);
    color: var(--fg);
    font:
      13px/1.5 -apple-system,
      system-ui,
      sans-serif;
  }
  main {
    max-width: 560px;
    margin: 0 auto;
    padding: 1.1rem 1.1rem 2rem;
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 0.9rem;
  }
  .brand h1 {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .brand span {
    color: var(--muted);
    font-size: 0.75rem;
  }
  .refresh {
    width: 30px;
    height: 30px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    color: var(--fg);
    font-size: 0.95rem;
    cursor: pointer;
  }
  .refresh:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.78rem;
    color: var(--muted);
    margin-bottom: 0.8rem;
    cursor: pointer;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }
  .empty-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    text-align: left;
    padding: 0.6rem 0.8rem;
    border: 1px dashed var(--line);
    border-radius: 10px;
    background: none;
    color: var(--muted);
    font-size: 0.82rem;
    cursor: pointer;
  }
  .empty-row:hover {
    color: var(--fg);
  }
  .empty-row .pip {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--line);
  }
  .empty-row .tag {
    margin-left: auto;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .err {
    color: var(--bad);
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
  }
  .muted {
    color: var(--muted);
  }
</style>
