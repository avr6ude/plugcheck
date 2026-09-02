<script lang="ts">
  import { onMount } from "svelte";
  import { savedCables, forgetCable, renameCable, type SavedCable } from "./snapshot.svelte";

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

<div class="sc">
  {#if rows.length === 0}
    <p class="empty">
      No saved cables yet. Name a cable or dock from its port card and it shows up here.
    </p>
  {/if}
  {#each rows as r}
    <div class="row">
      <div class="main">
        {#if editing === r.sig}
          <input
            bind:value={draft}
            placeholder="Name"
            onkeydown={(e) => e.key === "Enter" && commit(r.sig)}
            onblur={() => commit(r.sig)}
          />
        {:else}
          <button class="nm" onclick={() => start(r)}>
            {r.name ?? "Unnamed cable"}
          </button>
        {/if}
        <div class="meta">
          seen {r.count}× · first {r.first_seen} · last {r.last_seen}
          <span class="sig">{r.sig}</span>
        </div>
      </div>
      <button class="del" onclick={() => forget(r.sig)}>Forget</button>
    </div>
  {/each}
</div>

<style>
  .sc {
    display: grid;
    gap: 0.5rem;
  }
  .empty {
    color: var(--muted);
    font-size: 0.85rem;
    line-height: 1.5;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    border: 0.5px solid var(--line);
    border-radius: 10px;
    background: var(--card);
    padding: 0.6rem 0.8rem;
  }
  .main {
    flex: 1;
    min-width: 0;
  }
  .nm {
    font: 600 0.9rem/1.3 inherit;
    color: var(--fg);
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
  }
  .nm:hover {
    color: var(--accent);
  }
  input {
    font: 600 0.9rem/1.3 inherit;
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: 5px;
    padding: 0.1rem 0.35rem;
    width: 60%;
  }
  .meta {
    font-size: 0.72rem;
    color: var(--muted);
    margin-top: 0.15rem;
  }
  .sig {
    font-family: var(--mono);
    font-size: 0.66rem;
    display: block;
    opacity: 0.7;
    word-break: break-all;
  }
  .del {
    flex: none;
    background: none;
    border: 0.5px solid var(--line);
    border-radius: 6px;
    padding: 0.25rem 0.55rem;
    color: var(--muted);
    font-size: 0.75rem;
    cursor: pointer;
  }
  .del:hover {
    color: var(--bad);
    border-color: var(--bad);
  }
</style>
