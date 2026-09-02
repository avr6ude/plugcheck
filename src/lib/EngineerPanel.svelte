<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let { portId, onClose }: { portId: string; onClose: () => void } = $props();

  let rows = $state<Record<string, string>>({});
  let error = $state<string | null>(null);

  $effect(() => {
    invoke<Record<string, string>>("engineer_dump", { portId })
      .then((r) => (rows = r))
      .catch((e) => (error = String(e)));
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />
<!-- svelte-ignore a11y_click_events_have_key_events -- Escape (above) and the close button are the keyboard paths; the backdrop click is a mouse convenience -->
<div class="backdrop" onclick={onClose} role="presentation">
  <div
    class="panel"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-label="Engineer view"
    tabindex="-1"
  >
    <header>
      <strong>{portId}</strong>
      <button onclick={onClose}>close</button>
    </header>
    {#if error}
      <p class="err">{error}</p>
    {:else}
      <table>
        <tbody>
          {#each Object.entries(rows) as [k, v]}
            <tr><td>{k}</td><td>{v}</td></tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
  }
  .panel {
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: 10px;
    max-height: 80vh;
    width: 100%;
    max-width: 460px;
    overflow: auto;
    padding: 1rem;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.6rem;
  }
  button {
    font-size: 0.75rem;
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 6px;
    padding: 0.2rem 0.55rem;
    color: var(--muted);
    cursor: pointer;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--mono);
    font-size: 0.75rem;
  }
  td {
    border-top: 1px solid var(--line);
    padding: 0.25rem 0.4rem;
    vertical-align: top;
    word-break: break-word;
  }
  td:first-child {
    color: var(--muted);
    white-space: nowrap;
  }
  .err {
    color: #cf222e;
  }
</style>
