<script lang="ts">
  import type { Port, PortVerdict } from "./snapshot.svelte";
  import DeviceTree from "./DeviceTree.svelte";

  let {
    port,
    verdict,
    onEngineer,
  }: {
    port: Port;
    verdict: PortVerdict | undefined;
    onEngineer: (id: string) => void;
  } = $props();

  const dotClass = $derived(
    verdict?.data_blame === "none"
      ? "ok"
      : verdict?.data_blame === "cable"
        ? "warn"
        : verdict
          ? "bad"
          : "idle",
  );

  function emarkerSummary(): string {
    const e = port.emarker;
    if (!e.present) return "No e-marker";
    const bits = [
      e.vendor_name ?? (e.vendor_id != null ? `VID ${e.vendor_id}` : null),
      e.cable_type !== "unknown" ? e.cable_type : null,
      e.current_amps ? `${e.current_amps} A` : null,
      e.max_power_watts ? `${e.max_power_watts} W` : null,
    ].filter(Boolean);
    return bits.length ? bits.join(" · ") : "E-marked cable";
  }
</script>

<article class="card" class:empty={!port.occupied}>
  <header>
    <h2>{verdict?.headline ?? (port.occupied ? "Connected" : "Empty")}</h2>
    <span class="port-id">{port.id}</span>
  </header>

  {#if port.occupied}
    <p class="verdict">
      <span class="dot {dotClass}"></span>
      {verdict?.data_line ?? "…"}
    </p>

    {#if port.dp_alt}
      <p class="video">🖥 DisplayPort video active</p>
    {/if}

    {#if verdict?.charging_line}
      <p class="charging">⚡ {verdict.charging_line}</p>
    {/if}

    {#each verdict?.trust_flags ?? [] as flag}
      <p class="trust">⚠ {flag}</p>
    {/each}

    <p class="emarker">{emarkerSummary()}</p>

    {#if port.devices.length}
      <DeviceTree nodes={port.devices} />
    {/if}
  {/if}

  <button class="eng" onclick={() => onEngineer(port.id)}>Engineer</button>
</article>

<style>
  .card {
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 0.9rem 1rem;
    background: var(--card);
  }
  .card.empty {
    opacity: 0.55;
  }
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.5rem;
  }
  h2 {
    margin: 0;
    font-size: 1.05rem;
  }
  .port-id {
    color: var(--muted);
    font-size: 0.75rem;
    font-family: ui-monospace, monospace;
  }
  .verdict {
    margin: 0.55rem 0 0.3rem;
    font-size: 0.9rem;
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 0.45rem;
    vertical-align: middle;
  }
  .dot.ok {
    background: #2ea043;
  }
  .dot.warn {
    background: #d29922;
  }
  .dot.bad {
    background: #cf222e;
  }
  .dot.idle {
    background: var(--muted);
  }
  .charging,
  .video {
    margin: 0.2rem 0;
    font-size: 0.85rem;
  }
  .trust {
    margin: 0.2rem 0;
    font-size: 0.82rem;
    color: #d29922;
  }
  .emarker {
    margin: 0.4rem 0 0;
    font-size: 0.8rem;
    color: var(--muted);
  }
  .eng {
    margin-top: 0.7rem;
    font-size: 0.75rem;
    padding: 0.25rem 0.6rem;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
  }
  .eng:hover {
    color: var(--fg);
  }
</style>
