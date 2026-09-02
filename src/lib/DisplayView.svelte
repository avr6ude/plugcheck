<script lang="ts">
  import { store } from "./snapshot.svelte";

  const displays = $derived(
    (store.snapshot?.ports ?? [])
      .filter((p) => p.display)
      .map((p) => ({ port: p.id, d: p.display! })),
  );
</script>

<div class="dv">
  {#if displays.length === 0}
    <p class="empty">No external displays connected.</p>
  {/if}
  {#each displays as { port, d }}
    <div class="grp">
      <div class="gh">{d.name}<span>{port}</span></div>
      <dl>
        <div>
          <dt>Current mode</dt>
          <dd class:warn={d.degraded}>
            {d.pixels?.replace(" x ", " × ") ?? "—"}{d.hz ? ` @ ${d.hz} Hz` : ""}
          </dd>
        </div>
        {#if d.native_pixels}
          <div><dt>Native</dt><dd>{d.native_pixels.replace(" x ", " × ")}</dd></div>
        {/if}
        <div>
          <dt>Link</dt>
          <dd>{d.connection ?? "DisplayPort Alt Mode"}</dd>
        </div>
        {#if d.depth}<div><dt>Colour depth</dt><dd>{d.depth}</dd></div>{/if}
        <div><dt>HDR</dt><dd>{d.hdr ? "On" : "Off"}</dd></div>
        <div><dt>Role</dt><dd>{d.mirrored ? "Mirrored" : d.main ? "Main" : "Extended"}</dd></div>
      </dl>
      <p class="verdict" class:warn={d.degraded}>
        {#if d.degraded}
          Running below native. The link or macOS is limiting it — try a different
          cable/port or lower the refresh rate.
        {:else}
          Full quality{d.connection ? ` over ${d.connection}` : ""}.
        {/if}
      </p>
    </div>
  {/each}
</div>

<style>
  .dv {
    display: grid;
    gap: 1rem;
  }
  .empty {
    color: var(--muted);
  }
  .grp {
    border: 0.5px solid var(--line);
    border-radius: 10px;
    background: var(--card);
    padding: 0.7rem 0.85rem;
  }
  .gh {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-weight: 600;
    margin-bottom: 0.4rem;
  }
  .gh span {
    font: 0.7rem/1 var(--mono);
    color: var(--muted);
    font-weight: 400;
  }
  dl {
    margin: 0;
    display: grid;
    gap: 0.15rem;
  }
  dl > div {
    display: flex;
    gap: 0.7rem;
    font-size: 0.85rem;
  }
  dt {
    flex: none;
    width: 8rem;
    color: var(--muted);
  }
  dd {
    margin: 0;
  }
  dd.warn {
    color: var(--warn-fg);
    font-weight: 600;
  }
  .verdict {
    margin: 0.5rem 0 0;
    font-size: 0.82rem;
    color: var(--muted);
  }
  .verdict.warn {
    color: var(--warn-fg);
  }
</style>
