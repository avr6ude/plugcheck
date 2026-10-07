<script lang="ts">
  import { createAccordion, melt } from "@melt-ui/svelte";
  import type { DeviceNode } from "./snapshot.svelte";
  import Self from "./DeviceTree.svelte";

  let {
    nodes,
    depth = 0,
    forceOpen = false,
  }: { nodes: DeviceNode[]; depth?: number; forceOpen?: boolean } = $props();

  // Hubs start collapsed; `forceOpen` expands the lot.
  const accordion = createAccordion({ multiple: true });
  const { elements: { root, item, trigger, content }, helpers: { isSelected } } = accordion;
  $effect(() => accordion.options.forceVisible.set(forceOpen));

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

<ul class="tree" class:root={depth === 0} use:melt={$root}>
  {#each nodes as n, i}
    {@const key = `${depth}:${i}:${n.name}`}
    {@const hasKids = n.children.length > 0}
    {@const collapsible = n.is_hub && hasKids}
    {@const expanded = forceOpen || $isSelected(key)}
    <li use:melt={$item({ value: key })}>
      <div class="row">
        <button
          class="caret"
          class:open={expanded}
          class:hidden={!collapsible}
          use:melt={$trigger({ value: key })}
          aria-label={expanded ? "collapse" : "expand"}
        >
          <svg viewBox="0 0 12 12" width="9" height="9" aria-hidden="true">
            <path d="M4 2 L8 6 L4 10" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <div class="info">
          <div class="line1">
            <span class="name">{n.name}</span>
            {#if collapsible && !expanded}
              <span class="count">· {subtreeCount(n)} device{subtreeCount(n) === 1 ? "" : "s"}</span>
            {/if}
            {#if speedLabel(n.speed)}<span class="speed">{speedLabel(n.speed)}</span>{/if}
          </div>
          <div class="line2">
            {#if n.class}{n.class}{/if}
            {#if n.vendor}{n.class ? " · " : ""}{n.vendor}{/if}
            {#if n.usb_version}{" · " + n.usb_version}{/if}
            {#if n.vid_pid}<span class="mono"> · {n.vid_pid}</span>{/if}
            {#if n.serial}<span class="mono"> · SN {n.serial}</span>{/if}
          </div>
        </div>
      </div>
      {#if hasKids && n.is_hub}
        <div class="children" use:melt={$content({ value: key })}>
          <Self nodes={n.children} depth={depth + 1} {forceOpen} />
        </div>
      {:else if hasKids}
        <Self nodes={n.children} depth={depth + 1} {forceOpen} />
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
    align-items: flex-start;
    gap: 0.35rem;
    padding: 0.2rem 0;
  }
  .caret {
    width: 0.9rem;
    height: 1.34rem; /* == line1 line-box, so the glyph lands on the name's centre */
    flex: none;
    align-self: flex-start;
    background: none;
    border: 0;
    padding: 0;
    color: var(--muted);
    cursor: pointer;
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
  .info {
    min-width: 0;
    flex: 1;
  }
  .line1 {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
    font-size: 0.84rem;
    line-height: 1.6;
  }
  .name {
    font-weight: 500;
  }
  .count {
    color: var(--muted);
    font-size: 0.76rem;
  }
  .speed {
    margin-left: auto;
    color: var(--muted);
    font-size: 0.76rem;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .line2 {
    color: var(--muted);
    font-size: 0.72rem;
    line-height: 1.4;
  }
  .mono {
    font-family: var(--mono);
  }
</style>
