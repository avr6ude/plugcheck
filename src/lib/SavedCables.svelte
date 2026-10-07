<script lang="ts">
  import { onMount } from "svelte";
  import { savedCables, forgetCable, renameCable, settings, type SavedCable } from "./snapshot.svelte";

  let rows = $state<SavedCable[]>([]);
  let editing = $state<string | null>(null);
  let draft = $state("");

  async function load() {
    rows = await savedCables();
  }
  onMount(load);

  function start(r: SavedCable) {
    editing = r.sig;
    draft = r.name ?? "";
  }
  async function commit(sig: string) {
    editing = null;
    await renameCable(sig, draft.trim() || null);
    await load();
  }
  async function forget(sig: string) {
    await forgetCable(sig);
    await load();
  }
</script>

{#if rows.length === 0}
  <p class="empty-note">No saved cables yet. Name a cable on the Ports page and it shows up here.</p>
{:else}
  <ul class="group">
    {#each rows as r}
      <li class="row">
        <span class="main">
          {#if editing === r.sig}
            <input class="field" bind:value={draft} placeholder="Name" onkeydown={(e) => e.key === "Enter" && commit(r.sig)} onblur={() => commit(r.sig)} />
          {:else}
            <button class="name" title="Rename" onclick={() => start(r)}>{r.name ?? "Unnamed cable"}</button>
          {/if}
          <small>Seen {r.count} {r.count === 1 ? "time" : "times"} · first {r.first_seen} · last {r.last_seen}</small>
          {#if settings.show_technical}<small class="sig">{r.sig}</small>{/if}
        </span>
        <button class="push" onclick={() => forget(r.sig)}>Forget</button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .main { display: grid; gap: 2px; min-width: 0; }
  .name { justify-self: start; padding: 0; border: 0; background: none; text-align: left; }
  .field { width: 16rem; height: 22px; padding: 0 6px; border: 0; border-radius: 5px; background: var(--fill); -webkit-user-select: text; user-select: text; }
  small { color: var(--muted); font-size: 11px; }
  .sig { font-family: var(--mono); overflow-wrap: anywhere; -webkit-user-select: text; user-select: text; }
</style>
