<script lang="ts">
  import type { DeviceNode } from "./snapshot.svelte";
  import Self from "./DeviceTree.svelte";
  let { nodes }: { nodes: DeviceNode[] } = $props();

  function speedLabel(s: string): string {
    return s.replace(/_/g, " ").replace(/\bgen\b/, "Gen");
  }
</script>

<ul class="tree">
  {#each nodes as n}
    <li>
      <span class="dev">{n.is_hub ? "⎇ " : ""}{n.name}</span>
      {#if n.vendor}<span class="vendor">{n.vendor}</span>{/if}
      {#if n.speed !== "none"}<span class="speed">{speedLabel(n.speed)}</span>{/if}
      {#if n.children.length}<Self nodes={n.children} />{/if}
    </li>
  {/each}
</ul>

<style>
  .tree {
    list-style: none;
    margin: 0.4rem 0 0;
    padding-left: 0.9rem;
    border-left: 1px solid var(--line);
  }
  li {
    padding: 0.15rem 0;
    font-size: 0.85rem;
  }
  .dev {
    font-weight: 500;
  }
  .vendor,
  .speed {
    margin-left: 0.5rem;
    color: var(--muted);
    font-size: 0.78rem;
  }
</style>
