<script lang="ts">
  import { store } from "./snapshot.svelte";
  import { portLabel } from "./MacScene.svelte";

  const displays = $derived(
    (store.snapshot?.ports ?? [])
      .filter((p) => p.display)
      .map((p) => ({ port: portLabel(p), d: p.display! })),
  );
</script>

{#if displays.length === 0}
  <p class="empty-note">No external displays are connected.</p>
{/if}
{#each displays as { port, d }}
  <h3 class="list-title">{d.name}</h3>
  <dl class="group">
    <div class="row"><dt>Resolution</dt><dd>{#if d.degraded}<svg class="warn-mark" viewBox="0 0 20 20" width="14" height="14" aria-hidden="true"><circle cx="10" cy="10" r="10" /><path d="M10 5.6v5.3M10 14.3v.1" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" /></svg>{/if}{d.pixels?.replace(" x ", " × ") ?? "Unknown"}{d.hz ? ` at ${d.hz} Hz` : ""}</dd></div>
    {#if d.native_pixels}<div class="row"><dt>Native</dt><dd>{d.native_pixels.replace(" x ", " × ")}</dd></div>{/if}
    <div class="row"><dt>Connection</dt><dd>{d.connection ?? "DisplayPort Alt Mode"}</dd></div>
    {#if d.depth}<div class="row"><dt>Colour depth</dt><dd>{d.depth}</dd></div>{/if}
    <div class="row"><dt>HDR</dt><dd>{d.hdr ? "On" : "Off"}</dd></div>
    <div class="row"><dt>Arrangement</dt><dd>{d.mirrored ? "Mirrored" : d.main ? "Main display" : "Extended"}</dd></div>
    <div class="row"><dt>Port</dt><dd>{port}</dd></div>
  </dl>
  {#if d.degraded}
    <p class="group-note">Running below its native resolution. Try another cable or port, or a lower refresh rate.</p>
  {/if}
{/each}

<style>
  .warn-mark { flex: none; vertical-align: -2px; margin-right: 5px; } .warn-mark circle { fill: var(--warn); }
</style>
