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

  const NAV: { id: View; label: string; icon: string }[] = [
    { id: "ports", label: "Ports", icon: "▤" },
    { id: "power", label: "Power monitor", icon: "◠" },
    { id: "negotiation", label: "Negotiation", icon: "⇄" },
    { id: "displays", label: "Displays", icon: "▭" },
    { id: "cables", label: "Saved cables", icon: "❏" },
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

  <nav>
    {#each NAV as n}
      <button class:sel={view === n.id} onclick={() => onView(n.id)}>
        <span class="i">{n.icon}</span> {n.label}
      </button>
    {/each}
  </nav>

  <div class="sep"></div>

  <nav>
    <button onclick={() => refresh()}><span class="i">↻</span> Refresh now</button>
    <button onclick={onSettings}><span class="i">⚙</span> Settings…</button>
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
    width: 1.1rem;
    text-align: center;
    color: var(--muted);
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
