<script lang="ts">
  import { createTabs, melt } from "@melt-ui/svelte";
  import { refresh, store } from "./snapshot.svelte";

  type View = "ports" | "power" | "negotiation" | "displays" | "cables" | "settings";
  let { view, deviceCount, onView }: { view: View; deviceCount: number; onView: (v: View) => void } = $props();
  const NAV: { id: View; label: string }[] = [
    { id: "ports", label: "Ports" }, { id: "power", label: "Power Monitor" }, { id: "negotiation", label: "Negotiation" }, { id: "displays", label: "Displays" }, { id: "cables", label: "Saved Cables" },
  ];
  const { elements: { root, list, trigger } } = createTabs({ defaultValue: "ports", orientation: "vertical", activateOnFocus: true, onValueChange: ({ next }) => { onView(next as View); return next; } });
  // In the app the window paints the sidebar material; in a plain browser there is nothing behind it.
  const native = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
</script>

{#snippet icon(id: string)}
  <svg class="i" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
    {#if id === "ports"}<rect x="2" y="3.5" width="12" height="3.5" rx="1.75" /><rect x="2" y="9" width="12" height="3.5" rx="1.75" />
    {:else if id === "power"}<path d="M9.2 1.8 4.3 8.6h3.4l-.9 5.6 4.9-7H8.3z" />
    {:else if id === "negotiation"}<path d="M2.5 5.5h10M10 3l2.5 2.5L10 8M13.5 10.5h-10M6 8l-2.5 2.5L6 13" />
    {:else if id === "displays"}<rect x="1.8" y="2.5" width="12.4" height="8.6" rx="1.4" /><path d="M5.8 13.8h4.4M8 11.1v2.7" />
    {:else if id === "cables"}<path d="M4.3 2h7.4v12l-3.7-2.8L4.3 14z" />
    {:else if id === "refresh"}<path d="M13 8a5 5 0 1 1-1.47-3.54M13 2.3v3h-3" />
    {:else}<circle cx="8" cy="8" r="2.2" /><path d="M8 1.8v1.9M8 12.3v1.9M1.8 8h1.9M12.3 8h1.9M3.6 3.6 5 5M11 11l1.4 1.4M12.4 3.6 11 5M5 11l-1.4 1.4" />{/if}
  </svg>
{/snippet}

<aside class="sidebar" class:native aria-label="plugcheck" use:melt={$root}>
  <!-- Traffic lights live here; it also drags the window. -->
  <div class="titlebar" data-tauri-drag-region></div>
  <nav aria-label="Views" use:melt={$list}>
    {#each NAV as n}
      <button class="item" class:active={view === n.id} onclick={() => onView(n.id)} use:melt={$trigger(n.id)}>{@render icon(n.id)}<span>{n.label}</span></button>
    {/each}
  </nav>
  <nav class="tools" aria-label="Tools">
    <button class="item" onclick={() => refresh()}>{@render icon("refresh")}<span>Refresh Now</span></button>
    <button class="item" class:active={view === "settings"} onclick={() => onView("settings")}>{@render icon("settings")}<span>Settings</span></button>
  </nav>
  <footer>
    <img src="/favicon.png" alt="" width="20" height="20" />
    <span><b>plugcheck</b> {store.version}<br />{deviceCount} USB device{deviceCount === 1 ? "" : "s"}</span>
  </footer>
</aside>

<style>
  .sidebar { display: flex; flex-direction: column; min-height: 0; padding: 0 10px 12px; border-right: 1px solid var(--line); background: #e9e9e9; }
  .sidebar.native { background: transparent; }
  @media (prefers-color-scheme: dark) { .sidebar { background: #2a2a2a; } .sidebar.native { background: transparent; border-right-color: rgba(0, 0, 0, .5); } }
  .titlebar { flex: none; height: 52px; margin: 0 -10px; }
  nav { display: grid; gap: 1px; }
  .tools { margin-top: auto; }
  .item { display: flex; align-items: center; gap: 7px; height: 28px; width: 100%; padding: 0 8px; border: 0; border-radius: 6px; background: none; text-align: left; font-size: 13px; }
  .item.active { background: var(--selected); }
  .i { flex: none; fill: none; stroke: var(--accent); stroke-width: 1.4; stroke-linecap: round; stroke-linejoin: round; }
  footer { display: flex; align-items: center; gap: 8px; margin-top: 12px; padding: 10px 8px 0; border-top: 1px solid var(--line); color: var(--muted); font-size: 11px; line-height: 1.35; }
  footer b { color: var(--fg); font-weight: 600; }
</style>
