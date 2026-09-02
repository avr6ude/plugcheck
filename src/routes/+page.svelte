<script lang="ts">
  import { onMount } from "svelte";
  import {
    store,
    startPolling,
    refresh,
    verdictFor,
    settings,
    loadSettings,
    saveSettings,
  } from "$lib/snapshot.svelte";
  import PortCard from "$lib/PortCard.svelte";
  import EngineerPanel from "$lib/EngineerPanel.svelte";
  import SettingsPanel from "$lib/SettingsPanel.svelte";

  let engineerPort = $state<string | null>(null);
  let showSettings = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    loadSettings();
    startPolling().then((u) => (unlisten = u));
    return () => unlisten?.();
  });

  const ports = $derived(store.snapshot?.ports ?? []);
  const shown = $derived(
    settings.hide_empty ? ports.filter((p) => p.occupied) : ports,
  );
  const emptyCount = $derived(ports.filter((p) => !p.occupied).length);

  function toggleHideEmpty() {
    settings.hide_empty = !settings.hide_empty;
    saveSettings();
  }
</script>

<main>
  <header>
    <div class="brand">
      <h1>plugcheck</h1>
      <span>USB-C &amp; Thunderbolt inspector</span>
    </div>
    <div class="actions">
      <button onclick={() => (showSettings = true)} title="Settings" aria-label="Settings">⚙</button>
      <button onclick={refresh} disabled={store.loading} title="Refresh now">
        {store.loading ? "…" : "↻"}
      </button>
    </div>
  </header>

  {#if emptyCount > 0}
    <label class="toggle">
      <input type="checkbox" checked={settings.hide_empty} onchange={toggleHideEmpty} />
      Hide {emptyCount} empty port{emptyCount === 1 ? "" : "s"}
    </label>
  {/if}

  {#if store.error}
    <p class="err">{store.error}</p>
  {:else if store.snapshot}
    <div class="list">
      {#each shown as port, i (port.id)}
        {#if port.occupied}
          <div class="slot">
            <div class="slot-label">{port.kind} · Port {i + 1}</div>
            <PortCard
              {port}
              verdict={verdictFor(port.id)}
              onEngineer={(id) => (engineerPort = id)}
            />
          </div>
        {:else}
          <button class="empty-row" onclick={() => (engineerPort = port.id)}>
            <span class="pip"></span>
            <span class="lbl">{port.kind}</span>
            <span class="tag">Empty</span>
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

  {#if showSettings}
    <SettingsPanel onClose={() => (showSettings = false)} />
  {/if}
</main>

<style>
  :global(:root) {
    --bg: #f6f5f2;
    --fg: #201d19;
    --muted: #8a8378;
    --card: #fffdf9;
    --line: #e7e2d8;
    --accent: #0e8c7b;
    --ok: #1f9d55;
    --warn: #c17d0b;
    --warn-fg: #92600a;
    --bad: #d23b2f;
    --ui:
      "Avenir Next", "SF Pro Text", system-ui, -apple-system, sans-serif;
    --display: "New York", "Avenir Next", Georgia, serif;
    --mono: "SF Mono", "Menlo", ui-monospace, monospace;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #16161a;
      --fg: #eceae6;
      --muted: #8f897e;
      --card: #202024;
      --line: #33333a;
      --accent: #38c9b6;
      --ok: #3fcf7a;
      --warn: #e0a52e;
      --warn-fg: #e9be6a;
      --bad: #ff5c50;
    }
  }
  :global(body) {
    margin: 0;
    background: var(--bg);
    color: var(--fg);
    font: 13px/1.5 var(--ui);
    -webkit-font-smoothing: antialiased;
    letter-spacing: 0.005em;
  }
  main {
    max-width: 660px;
    margin: 0 auto;
    padding: 1.2rem 1.2rem 2.5rem;
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 0.9rem;
  }
  .brand h1 {
    margin: 0;
    font-family: var(--display);
    font-size: 1.4rem;
    font-weight: 600;
    letter-spacing: 0;
  }
  .brand span {
    color: var(--muted);
    font-size: 0.75rem;
  }
  .actions {
    display: flex;
    gap: 0.4rem;
  }
  .actions button {
    width: 30px;
    height: 30px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    color: var(--fg);
    font-size: 0.95rem;
    cursor: pointer;
  }
  .actions button:disabled {
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
    gap: 1.1rem;
  }
  .slot-label {
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted);
    margin: 0 0 0.35rem 0.9rem;
  }
  .empty-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    text-align: left;
    padding: 0.7rem 0.9rem;
    border: 0.5px solid var(--line);
    border-radius: 12px;
    background: var(--card);
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
  }
  .err {
    color: var(--bad);
    font-family: var(--mono);
    font-size: 0.85rem;
  }
  .muted {
    color: var(--muted);
  }
</style>
