<script lang="ts">
  import type { DeviceNode } from "./snapshot.svelte";
  import Self from "./DeviceTree.svelte";

  let { nodes, depth = 0 }: { nodes: DeviceNode[]; depth?: number } = $props();

  // Hubs start collapsed (WhatCable does the same).
  let open = $state<Record<string, boolean>>({});

  function speedLabel(s: string): string {
    const map: Record<string, string> = {
      none: "",
      usb2: "USB 2.0",
      usb3_gen1: "5 Gb/s",
      usb3_gen2: "10 Gb/s",
      usb4_gen3: "20 Gb/s",
      usb4_gen4: "40 Gb/s",
      thunderbolt3: "TB3",
      thunderbolt4: "TB4",
      displayport: "DP",
    };
    return map[s] ?? s;
  }

  function subtreeCount(n: DeviceNode): number {
    return n.children.reduce((a, c) => a + 1 + subtreeCount(c), 0);
  }
</script>

<ul class="tree" class:root={depth === 0}>
  {#each nodes as n, i}
    {@const key = `${depth}:${i}:${n.name}`}
    {@const hasKids = n.children.length > 0}
    {@const collapsible = n.is_hub && hasKids}
    <li>
      <div class="row">
        <button
          class="caret"
          class:open={open[key]}
          class:hidden={!collapsible}
          onclick={() => (open[key] = !open[key])}
          aria-label={open[key] ? "collapse" : "expand"}
        >
          <svg viewBox="0 0 12 12" width="9" height="9" aria-hidden="true">
            <path d="M4 2 L8 6 L4 10" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <span class="name">{n.name}</span>
        {#if collapsible && !open[key]}
          <span class="count">{subtreeCount(n)} device{subtreeCount(n) === 1 ? "" : "s"}</span>
        {/if}
        {#if n.class}<span class="tag">{n.class}</span>{/if}
        {#if n.vendor}<span class="dim">{n.vendor}</span>{/if}
        {#if n.usb_version}<span class="dim">{n.usb_version}</span>{/if}
        {#if n.vid_pid}<span class="mono">{n.vid_pid}</span>{/if}
        {#if speedLabel(n.speed)}<span class="speed">{speedLabel(n.speed)}</span>{/if}
      </div>
      {#if hasKids && (open[key] || !n.is_hub)}
        <Self nodes={n.children} depth={depth + 1} />
      {/if}
    </li>
  {/each}
</ul>

<style>
  .tree {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .tree:not(.root) {
    margin-left: 1.15rem;
    border-left: 1px solid var(--line);
    padding-left: 0.35rem;
  }
  li {
    padding: 0.05rem 0;
  }
  .row {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
    font-size: 0.84rem;
    line-height: 1.75;
  }
  .caret {
    width: 0.9rem;
    flex: none;
    background: none;
    border: 0;
    padding: 0;
    color: var(--muted);
    cursor: pointer;
    align-self: center;
    display: grid;
    place-items: center;
    transition: transform 0.12s ease;
  }
  .caret.open {
    transform: rotate(90deg);
  }
  .caret.hidden {
    visibility: hidden;
    cursor: default;
  }
  .name {
    font-weight: 500;
  }
  .count,
  .dim,
  .speed,
  .mono {
    color: var(--muted);
    font-size: 0.76rem;
  }
  .mono {
    font-family: ui-monospace, monospace;
    font-size: 0.72rem;
  }
  .tag {
    font-size: 0.68rem;
    padding: 0.02rem 0.32rem;
    border: 1px solid var(--line);
    border-radius: 4px;
    color: var(--muted);
  }
  .speed {
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }
</style>
