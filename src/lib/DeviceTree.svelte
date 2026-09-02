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
    <li>
      <div class="row">
        {#if n.is_hub && hasKids}
          <button class="twist" onclick={() => (open[key] = !open[key])} aria-label="toggle">
            {open[key] ? "▾" : "▸"}
          </button>
        {:else}
          <span class="twist spacer"></span>
        {/if}
        <span class="name">{n.name}</span>
        {#if n.is_hub && hasKids && !open[key]}
          <span class="count">{subtreeCount(n)} device{subtreeCount(n) === 1 ? "" : "s"}</span>
        {/if}
        {#if n.vendor}<span class="vendor">{n.vendor}</span>{/if}
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
    margin-left: 1.1rem;
    border-left: 1px solid var(--line);
    padding-left: 0.3rem;
  }
  li {
    padding: 0.05rem 0;
  }
  .row {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
    font-size: 0.84rem;
    line-height: 1.7;
  }
  .twist {
    width: 1rem;
    flex: none;
    background: none;
    border: 0;
    padding: 0;
    color: var(--muted);
    cursor: pointer;
    font-size: 0.7rem;
  }
  .twist.spacer {
    cursor: default;
  }
  .name {
    font-weight: 500;
  }
  .count,
  .vendor,
  .speed {
    color: var(--muted);
    font-size: 0.76rem;
  }
  .speed {
    margin-left: auto;
    font-variant-numeric: tabular-nums;
  }
</style>
