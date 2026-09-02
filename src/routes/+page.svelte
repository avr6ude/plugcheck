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
    --bg: #f2f2f7;
    --fg: #1c1c1e;
    --muted: #8a8a8e;
    --card: #ffffff;
    --line: #d8d8dc;
    --accent: #0a84ff;
    --ok: #34c759;
    --warn: #ff9f0a;
    --warn-fg: #a8690a;
    --bad: #ff3b30;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #1c1c1e;
      --fg: #f2f2f7;
      --muted: #8e8e93;
      --card: #2c2c2e;
      --line: #3a3a3c;
      --accent: #0a84ff;
      --ok: #30d158;
      --warn: #ff9f0a;
      --warn-fg: #ffd60a;
      --bad: #ff453a;
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
    -webkit-font-smoothing: antialiased;
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
    font-size: 1.15rem;
    font-weight: 650;
    letter-spacing: -0.01em;
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
    font-family: ui-monospace, monospace;
    font-size: 0.85rem;
  }
  .muted {
    color: var(--muted);
  }
</style>
