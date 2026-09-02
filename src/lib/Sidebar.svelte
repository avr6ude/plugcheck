<script lang="ts">
  import { settings, saveSettings, refresh, store } from "./snapshot.svelte";

  type View = "ports" | "power" | "negotiation" | "displays" | "cables";

  let {
    open,
    view,
    technical,
    deviceCount,
    onClose,
    onView,
    onSettings,
    onToggleTechnical,
  }: {
    open: boolean;
    view: View;
    technical: boolean;
    deviceCount: number;
    onClose: () => void;
    onView: (v: View) => void;
    onSettings: () => void;
    onToggleTechnical: () => void;
  } = $props();

  const NAV: { id: View; label: string }[] = [
    { id: "ports", label: "Ports" },
    { id: "power", label: "Power monitor" },
    { id: "negotiation", label: "Negotiation" },
    { id: "displays", label: "Displays" },
    { id: "cables", label: "Saved cables" },
  ];

  function toggleEmpty() {
    settings.hide_empty = !settings.hide_empty;
    saveSettings();
  }
</script>

{#if open}
  <div class="scrim" onclick={onClose} role="presentation"></div>
{/if}

<aside class="bar" class:open>
  <div class="brand">
    <span class="plug" aria-hidden="true">
      <svg viewBox="0 0 20 20" width="18" height="18">
        <rect x="3.5" y="6" width="10" height="8" rx="4" fill="none" stroke="currentColor" stroke-width="1.6"/>
        <path d="M13.5 8.5 h3 M13.5 11.5 h3" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
      </svg>
    </span>
    <b>plugcheck</b>
  </div>

  {#snippet icon(id: string)}
    <svg class="i" viewBox="0 0 16 16" width="15" height="15" aria-hidden="true">
      {#if id === "ports"}
        <rect x="2" y="3" width="12" height="3" rx="1" /><rect x="2" y="8.5" width="12" height="3" rx="1" />
      {:else if id === "power"}
        <path d="M9 1.5 L4 8.5 H7.5 L7 14.5 L12 7 H8.5 Z" fill="currentColor" stroke="none" />
      {:else if id === "negotiation"}
        <path d="M3 5 h9 M9.5 2.5 L12.5 5 L9.5 7.5 M13 11 h-9 M6.5 8.5 L3.5 11 L6.5 13.5" />
      {:else if id === "displays"}
        <rect x="1.5" y="2.5" width="13" height="9" rx="1.2" /><path d="M6 14 h4" stroke-linecap="round" />
      {:else if id === "cables"}
        <path d="M4 1.8 h8 v12.4 l-4 -3 -4 3 Z" stroke-linejoin="round" />
      {:else if id === "refresh"}
        <path d="M13 8 a5 5 0 1 1 -1.5 -3.5 M13 2 v3 h-3" stroke-linecap="round" stroke-linejoin="round" />
      {:else if id === "settings"}
        <circle cx="8" cy="8" r="2.3" /><path d="M8 1.5 v2 M8 12.5 v2 M1.5 8 h2 M12.5 8 h2 M3.5 3.5 l1.4 1.4 M11.1 11.1 l1.4 1.4 M12.5 3.5 l-1.4 1.4 M4.9 11.1 l-1.4 1.4" stroke-linecap="round" />
      {/if}
    </svg>
  {/snippet}

  <nav>
    {#each NAV as n}
      <button class:sel={view === n.id} onclick={() => onView(n.id)}>
        {@render icon(n.id)} {n.label}
      </button>
    {/each}
  </nav>

  <div class="sep"></div>

  <nav>
    <button onclick={() => refresh()}>{@render icon("refresh")} Refresh now</button>
    <button onclick={onSettings}>{@render icon("settings")} Settings…</button>
    <button class="chk" onclick={onToggleTechnical}>
      <span class="box" class:on={technical}>{technical ? "✓" : ""}</span>
      Show technical details
    </button>
    <button class="chk" onclick={toggleEmpty}>
      <span class="box" class:on={settings.hide_empty}>{settings.hide_empty ? "✓" : ""}</span>
      Hide empty ports
    </button>
  </nav>

  <div class="foot">
    <div>{deviceCount} USB device{deviceCount === 1 ? "" : "s"}</div>
    <div>plugcheck {store.version}</div>
    <div class="hint">CLI: <code>plugcheck --text</code></div>
  </div>
</aside>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.25);
    z-index: 40;
  }
  .bar {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 230px;
    background: var(--card);
    border-right: 0.5px solid var(--line);
    transform: translateX(-100%);
    transition: transform 0.16s ease;
    z-index: 50;
    display: flex;
    flex-direction: column;
    padding: 0.9rem 0.7rem;
  }
  .bar.open {
    transform: translateX(0);
    box-shadow: 2px 0 20px rgba(0, 0, 0, 0.15);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0 0.35rem 0.7rem;
    font-size: 1rem;
  }
  .plug {
    color: var(--accent);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  nav button {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    width: 100%;
    text-align: left;
    background: none;
    border: 0;
    border-radius: 6px;
    padding: 0.45rem 0.4rem;
    color: var(--fg);
    font-size: 0.84rem;
    cursor: pointer;
  }
  nav button:hover {
    background: color-mix(in srgb, var(--muted) 12%, transparent);
  }
  nav button.sel {
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    color: color-mix(in srgb, var(--accent) 70%, var(--fg));
    font-weight: 600;
  }
  .i {
    flex: none;
    color: var(--muted);
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
  }
  nav button.sel .i {
    color: inherit;
  }
  .sep {
    height: 0.5px;
    background: var(--line);
    margin: 0.5rem 0;
  }
  .box {
    width: 15px;
    height: 15px;
    border: 1px solid var(--line);
    border-radius: 4px;
    display: grid;
    place-items: center;
    font-size: 0.68rem;
    color: #fff;
  }
  .box.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .foot {
    margin-top: auto;
    border-top: 0.5px solid var(--line);
    padding: 0.6rem 0.4rem 0;
    font-size: 0.72rem;
    color: var(--muted);
    display: grid;
    gap: 0.2rem;
  }
  .foot code {
    font-family: var(--mono);
    font-size: 0.68rem;
  }
</style>
