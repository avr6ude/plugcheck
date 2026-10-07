<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { store, startPolling, refresh, verdictFor, settings, loadSettings } from "$lib/snapshot.svelte";
  import SettingsPanel from "$lib/SettingsPanel.svelte";
  import Sidebar from "$lib/Sidebar.svelte";
  import PowerMonitor from "$lib/PowerMonitor.svelte";
  import NegotiationView from "$lib/NegotiationView.svelte";
  import DisplayView from "$lib/DisplayView.svelte";
  import SavedCables from "$lib/SavedCables.svelte";
  import MacScene, { sidePorts } from "$lib/MacScene.svelte";
  import PortInspector from "$lib/PortInspector.svelte";

  type View = "ports" | "power" | "negotiation" | "displays" | "cables" | "settings";
  let view = $state<View>("ports");
  let selectedPortId = $state<string | null>(null);
  let side = $state<"left" | "right">("right");

  onMount(() => {
    let unlisten: (() => void) | undefined;
    loadSettings();
    startPolling().then((u) => (unlisten = u));
    const active = () => document.documentElement.classList.toggle("inactive", !document.hasFocus());
    // ⌘R rescans (instead of reloading the webview), ⌘, opens Settings.
    const keys = (e: KeyboardEvent) => {
      if (!e.metaKey) return;
      if (e.key === "r") { e.preventDefault(); refresh(); }
      if (e.key === ",") { e.preventDefault(); view = "settings"; }
    };
    active();
    addEventListener("focus", active); addEventListener("blur", active); addEventListener("keydown", keys);
    // Menu-bar menu: Refresh and Settings… arrive here.
    const offMenu = listen<string>("menu", (ev) => (ev.payload === "settings" ? (view = "settings") : refresh())).catch(() => undefined);
    return () => { unlisten?.(); offMenu.then((off) => off?.()); removeEventListener("focus", active); removeEventListener("blur", active); removeEventListener("keydown", keys); };
  });

  const ports = $derived(store.snapshot?.ports ?? []);
  const deviceCount = $derived(ports.reduce((a, p) => {
    const walk = (nodes: typeof p.devices): number => nodes.reduce((x, d) => x + 1 + walk(d.children), 0);
    return a + walk(p.devices);
  }, 0));
  const sideEntries = $derived(sidePorts(ports, side));
  const sideConnectedCount = $derived(sideEntries.filter((e) => e.port?.occupied).length);
  const defaultSidePort = $derived(sideEntries.find((e) => e.port?.occupied)?.port ?? sideEntries.find((e) => e.port)?.port);
  const selectedPort = $derived(ports.find((p) => p.id === selectedPortId) ?? (selectedPortId ? undefined : defaultSidePort));
  const selectedVerdict = $derived(selectedPort ? verdictFor(selectedPort.id) : undefined);
  const fallbackPort = $derived(sideEntries.find((e) => e.fallbackId === selectedPortId));
  const viewTitle = $derived({ ports: "Ports", power: "Power Monitor", negotiation: "Negotiation", displays: "Displays", cables: "Saved Cables", settings: "Settings" }[view]);
  const subtitle = $derived(view === "ports" ? `${sideConnectedCount} of ${sideEntries.length} in use on the ${side} side` : "");
</script>

<div class="window">
  <Sidebar {view} {deviceCount} onView={(v) => (view = v)} />

  <div class="content">
    <!-- Toolbar sits in the title-bar zone, where macOS keeps the clicks: title text only, and it drags the window. -->
    <header class="toolbar" data-tauri-drag-region="deep">
      <h1>{viewTitle}</h1>
      {#if subtitle}<p>{subtitle}</p>{/if}
    </header>

    <main class:ports={view === "ports"}>
      {#if view === "ports"}
        <section class="model-column" aria-label="MacBook port selector">
          <MacScene {ports} selectedPortId={selectedPort?.id ?? selectedPortId} {side} onSelectPort={(id) => (selectedPortId = id)} onSideChange={(next) => { side = next; selectedPortId = null; }} />
        </section>
        <section class="inspector-column" aria-label="Selected port details">
          {#if selectedPort}
            <PortInspector port={selectedPort} verdict={selectedVerdict} technical={settings.show_technical} />
          {:else}
            <div class="placeholder" role={store.error ? "alert" : undefined}>
              {#if store.error}
                <h2>Couldn’t read your ports</h2>
                <p>{store.error}</p>
              {:else if !store.snapshot}
                <h2>Reading your ports…</h2>
              {:else}
                <h2>{fallbackPort ? `Nothing is connected to ${fallbackPort.label}.` : "No port selected"}</h2>
                {#if !fallbackPort}<p>Choose a port next to the Mac.</p>{/if}
              {/if}
              {#if store.error}<button class="push" onclick={() => refresh()}>Try Again</button>{/if}
            </div>
          {/if}
        </section>
      {:else if view === "settings"}
        <div class="pane"><SettingsPanel /></div>
      {:else if store.error}
        <div class="placeholder" role="alert">
          <h2>Couldn’t read your ports</h2>
          <p>{store.error}</p>
          <button class="push" onclick={() => refresh()}>Try Again</button>
        </div>
      {:else if !store.snapshot}
        <div class="placeholder"><h2>Reading your ports…</h2></div>
      {:else}
        <div class="pane">
          {#if view === "power"}<PowerMonitor />
          {:else if view === "negotiation"}<NegotiationView />
          {:else if view === "displays"}<DisplayView />
          {:else if view === "cables"}<SavedCables />{/if}
        </div>
      {/if}
    </main>
  </div>
</div>

<style>
  /* macOS semantic colours, light first; the app follows the system appearance. */
  :global(:root) {
    color-scheme: light dark;
    --ui: -apple-system, BlinkMacSystemFont, "Helvetica Neue", sans-serif;
    --mono: ui-monospace, "SF Mono", Menlo, monospace;
    --accent: #007aff;
    --bg: #f5f5f5; --group: #ffffff;
    --fg: rgba(0, 0, 0, .85); --muted: rgba(0, 0, 0, .5); --tertiary: rgba(0, 0, 0, .26);
    --line: rgba(0, 0, 0, .1); --fill: rgba(0, 0, 0, .05); --hover: rgba(0, 0, 0, .04); --selected: rgba(0, 0, 0, .1);
    --btn: #ffffff; --btn-edge: 0 0 0 .5px rgba(0, 0, 0, .14), 0 1px 1.5px rgba(0, 0, 0, .1);
    --ok: #28cd41; --warn: #ff9500; --bad: #ff3b30;
    /* names the tool views still use */
    --card: var(--group); --card-strong: var(--fill); --warn-fg: var(--warn);
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --accent: #0a84ff;
      --bg: #1e1e1e; --group: #282828;
      --fg: rgba(255, 255, 255, .85); --muted: rgba(255, 255, 255, .55); --tertiary: rgba(255, 255, 255, .25);
      --line: rgba(255, 255, 255, .1); --fill: rgba(255, 255, 255, .08); --hover: rgba(255, 255, 255, .05); --selected: rgba(255, 255, 255, .1);
      --btn: rgba(255, 255, 255, .17); --btn-edge: 0 0 0 .5px rgba(0, 0, 0, .3), inset 0 .5px 0 rgba(255, 255, 255, .12);
      --ok: #32d74b; --warn: #ff9f0a; --bad: #ff453a;
    }
  }
  @supports (color: AccentColor) { :global(:root) { --accent: AccentColor; } }

  :global(*) { box-sizing: border-box; }
  /* Transparent so the window's sidebar material shows through; the content pane paints its own ground. */
  :global(html), :global(body) { margin: 0; height: 100%; overflow: hidden; overscroll-behavior: none; background: transparent; }
  :global(body) { color: var(--fg); font: 13px/1.4 var(--ui); -webkit-font-smoothing: antialiased; user-select: none; -webkit-user-select: none; cursor: default; }
  :global(button), :global(input), :global(select) { font: inherit; color: inherit; }
  :global(::selection) { background: color-mix(in srgb, var(--accent) 30%, transparent); }
  :global(:focus-visible) { outline: 3px solid color-mix(in srgb, var(--accent) 55%, transparent); outline-offset: 0; }
  :global(.push) { height: 22px; padding: 0 12px; border: 0; border-radius: 5px; background: var(--btn); box-shadow: var(--btn-edge); font-size: 13px; }
  :global(.push:active) { filter: brightness(.94); }
  /* Grouped inset list (System Settings): the one container every view uses. */
  :global(.list-title) { margin: 20px 0 6px 12px; font-size: 13px; font-weight: 600; }
  :global(.list-title:first-child) { margin-top: 0; }
  :global(.group) { margin: 0; padding: 0 0 0 12px; list-style: none; border-radius: 10px; background: var(--group); box-shadow: 0 0 0 .5px var(--line); }
  :global(.group > .row) { display: flex; align-items: center; justify-content: space-between; gap: 16px; min-height: 34px; padding: 7px 12px 7px 0; }
  :global(.group > .row + .row) { border-top: 1px solid var(--line); }
  :global(.group dt) { flex: none; }
  :global(.group dd) { margin: 0; min-width: 0; color: var(--muted); text-align: right; overflow-wrap: anywhere; font-variant-numeric: tabular-nums; -webkit-user-select: text; user-select: text; cursor: text; }
  :global(.group-note) { margin: 6px 12px 0; color: var(--muted); font-size: 11px; }
  :global(.empty-note) { margin: 48px auto; max-width: 26rem; color: var(--muted); text-align: center; }
  .pane { max-width: 640px; margin: 0 auto; }

  .window { display: grid; grid-template-columns: 200px minmax(0, 1fr); grid-template-rows: minmax(0, 1fr); height: 100vh; }
  .content { display: flex; flex-direction: column; min-width: 0; background: var(--bg); }
  .toolbar { flex: none; display: flex; flex-direction: column; justify-content: center; height: 52px; padding: 0 20px; border-bottom: 1px solid var(--line); }
  h1 { margin: 0; font-size: 15px; font-weight: 700; line-height: 1.2; }
  .toolbar p { margin: 1px 0 0; color: var(--muted); font-size: 11px; }
  main { flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: none; padding: 24px; }
  /* One row pinned to the pane height: the details column scrolls, the Mac stays put. */
  main.ports { display: grid; grid-template-rows: minmax(0, 1fr); grid-template-columns: minmax(0, 1.5fr) minmax(320px, 1fr); gap: 24px; padding: 0 24px 0 12px; overflow: hidden; }
  .model-column { display: grid; min-width: 0; min-height: 0; padding-top: 14px; }
  .inspector-column { min-width: 0; min-height: 0; padding-top: 20px; }
  .placeholder { display: grid; justify-items: center; gap: 6px; align-content: center; min-height: 60%; text-align: center; }
  .placeholder h2 { margin: 0; font-size: 17px; font-weight: 600; }
  .placeholder p { margin: 0 0 6px; max-width: 26rem; color: var(--muted); }

  /* Narrower windows stack the model over the details; the whole pane scrolls. */
  @media (max-width: 1099px) {
    main.ports { grid-template-rows: none; grid-template-columns: minmax(0, 1fr); gap: 0; padding: 0 24px; overflow-y: auto; }
    .model-column { height: 260px; }
    .inspector-column { border-top: 1px solid var(--line); }
    .inspector-column :global(.scroll) { overflow: visible; }
  }
</style>
