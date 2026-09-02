<script lang="ts">
  import { onMount } from "svelte";
  import {
    store,
    startPolling,
    verdictFor,
    settings,
    loadSettings,
  } from "$lib/snapshot.svelte";
  import PortCard from "$lib/PortCard.svelte";
  import EngineerPanel from "$lib/EngineerPanel.svelte";
  import SettingsPanel from "$lib/SettingsPanel.svelte";
  import Sidebar from "$lib/Sidebar.svelte";
  import PowerMonitor from "$lib/PowerMonitor.svelte";
  import NegotiationView from "$lib/NegotiationView.svelte";
  import DisplayView from "$lib/DisplayView.svelte";
  import SavedCables from "$lib/SavedCables.svelte";

  type View = "ports" | "power" | "negotiation" | "displays" | "cables";

  let engineerPort = $state<string | null>(null);
  let showSettings = $state(false);
  let sidebar = $state(false);
  let technical = $state(false);
  let view = $state<View>("ports");

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
  const deviceCount = $derived(
    ports.reduce((a, p) => {
      const w = (ns: typeof p.devices): number =>
        ns.reduce((x, d) => x + 1 + w(d.children), 0);
      return a + w(p.devices);
    }, 0),
  );
</script>

<Sidebar
  open={sidebar}
  {view}
  {technical}
  {deviceCount}
  onClose={() => (sidebar = false)}
  onView={(v) => {
    view = v;
    sidebar = false;
  }}
  onSettings={() => {
    sidebar = false;
    showSettings = true;
  }}
  onToggleTechnical={() => (technical = !technical)}
/>

<div class="bar">
  <button class="ham" onclick={() => (sidebar = true)} aria-label="Menu">
    <svg viewBox="0 0 16 16" width="15" height="15"
      ><path d="M2 4h12M2 8h12M2 12h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
    /></svg>
  </button>
  <span class="wm">plugcheck</span>
  {#if store.loading}<span class="load">…</span>{/if}
</div>

<main>
  {#if store.error}
    <p class="err">{store.error}</p>
  {:else if !store.snapshot}
    <p class="muted">Reading ports…</p>
  {:else if view === "power"}
    <PowerMonitor />
  {:else if view === "negotiation"}
    <NegotiationView />
  {:else if view === "displays"}
    <DisplayView />
  {:else if view === "cables"}
    <SavedCables />
  {:else}
    <div class="list">
      {#each shown as port, i (port.id)}
        {#if port.occupied}
          <div class="slot">
            <div class="slot-label">{port.kind} · Port {i + 1}</div>
            <PortCard
              {port}
              verdict={verdictFor(port.id)}
              {technical}
              onEngineer={(id) => (engineerPort = id)}
            />
          </div>
        {:else}
          <button class="empty-row" onclick={() => (engineerPort = port.id)}>
            {port.kind}<span class="tag">Empty</span>
          </button>
        {/if}
      {/each}
    </div>
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
    --ui: -apple-system, system-ui, "Helvetica Neue", Arial, sans-serif;
    --mono: ui-monospace, "SF Mono", Menlo, monospace;
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
  }

  .bar {
    position: sticky;
    top: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.9rem;
    background: color-mix(in srgb, var(--bg) 86%, transparent);
    backdrop-filter: blur(8px);
    border-bottom: 0.5px solid var(--line);
  }
  .ham {
    background: none;
    border: 0;
    padding: 0.2rem;
    color: var(--fg);
    cursor: pointer;
    display: grid;
    place-items: center;
  }
  .wm {
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .load {
    margin-left: auto;
    color: var(--muted);
  }

  main {
    max-width: 640px;
    margin: 0 auto;
    padding: 1rem 1rem 1.5rem;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }
  .slot-label {
    font-size: 0.64rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--muted);
    margin: 0 0 0.3rem 0.15rem;
  }
  .empty-row {
    display: flex;
    align-items: center;
    width: 100%;
    text-align: left;
    padding: 0.6rem 0.9rem;
    border: 0.5px dashed var(--line);
    border-radius: 10px;
    background: none;
    color: var(--muted);
    font-size: 0.82rem;
    cursor: pointer;
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
