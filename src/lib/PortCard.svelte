<script lang="ts">
  import type { Port, PortVerdict, CardStatus } from "./snapshot.svelte";
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

  const glyph: Record<CardStatus, string> = {
    ok: "✓",
    warn: "!",
    bad: "✕",
    idle: "·",
  };

  // Worst card status → the dot next to the headline.
  const rank: Record<CardStatus, number> = { idle: 0, ok: 1, warn: 2, bad: 3 };
  const worst = $derived(
    (verdict?.cards ?? []).reduce<CardStatus>(
      (w, c) => (rank[c.status] > rank[w] ? c.status : w),
      "ok",
    ),
  );

  function deviceCount(): number {
    const walk = (n: Port["devices"]): number =>
      n.reduce((a, d) => a + 1 + walk(d.children), 0);
    return walk(port.devices);
  }

  const cableKind = $derived(
    { passive: "Passive", active: "Active", optical: "Optical" }[port.cable_kind] ?? null,
  );

  // key/value rows for the collapsible "Port details"
  const details = $derived(
    (
      [
        ["Cable", cableKind],
        ["Orientation", port.orientation != null ? `Position ${port.orientation}` : null],
        ["Negotiated", port.provisioned.length ? port.provisioned.join(", ") : null],
        ["Display hot-plug", port.hpd ? "Detected" : null],
        [
          "Connections",
          port.connection_count != null ? `${port.connection_count} since boot` : null,
        ],
        [
          "Plug events",
          port.plug_events != null ? `${port.plug_events} since boot` : null,
        ],
        [
          "Overcurrent",
          port.overcurrent_count != null ? `${port.overcurrent_count} recorded` : null,
        ],
      ] as [string, string | null][]
    ).filter(([, v]) => v != null) as [string, string][],
  );
</script>

<section class="port">
  <header>
    <span class="pip {worst}"></span>
    <h2>{verdict?.headline ?? (port.occupied ? "Connected" : "Empty")}</h2>
    <span class="meta">{port.kind}</span>
  </header>

  {#each verdict?.cards ?? [] as card}
    <div class="card {card.status}">
      <span class="badge {card.status}">{glyph[card.status]}</span>
      <div class="body">
        <span class="title">{card.title}</span>
        <p>{card.text}</p>
      </div>
    </div>
  {/each}

  {#each verdict?.trust_flags ?? [] as flag}
    <div class="card warn">
      <span class="badge warn">!</span>
      <div class="body">
        <span class="title">Trust</span>
        <p>{flag}</p>
      </div>
    </div>
  {/each}

  {#if port.devices.length}
    <div class="devices">
      <span class="dev-head">Connected devices ({deviceCount()})</span>
      <DeviceTree nodes={port.devices} />
    </div>
  {/if}

  {#if details.length}
    <details class="more">
      <summary>Port details</summary>
      <dl>
        {#each details as [k, v]}
          <div><dt>{k}</dt><dd>{v}</dd></div>
        {/each}
      </dl>
    </details>
  {/if}

  <button class="eng" onclick={() => onEngineer(port.id)}>Raw IOKit data ›</button>
</section>

<style>
  .port {
    padding: 0.95rem 1rem 0.75rem;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--card);
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    margin-bottom: 0.7rem;
  }
  .pip {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex: none;
    align-self: center;
  }
  h2 {
    margin: 0;
    font-size: 0.98rem;
    font-weight: 600;
  }
  .meta {
    margin-left: auto;
    color: var(--muted);
    font-size: 0.74rem;
    font-family: ui-monospace, monospace;
  }

  .card {
    display: flex;
    gap: 0.6rem;
    padding: 0.5rem 0;
    border-top: 1px solid var(--line);
  }
  .card:first-of-type {
    border-top: 0;
  }
  .badge {
    flex: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 0.7rem;
    font-weight: 700;
    color: #fff;
    margin-top: 0.1rem;
  }
  .badge.ok {
    background: var(--ok);
  }
  .badge.warn {
    background: var(--warn);
  }
  .badge.bad {
    background: var(--bad);
  }
  .badge.idle {
    background: var(--muted);
  }
  .body {
    min-width: 0;
  }
  .title {
    display: block;
    font-size: 0.68rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .body p {
    margin: 0.1rem 0 0;
    font-size: 0.86rem;
    line-height: 1.45;
  }
  .card.warn .body p {
    color: var(--warn-fg);
  }

  .devices {
    margin-top: 0.6rem;
    padding-top: 0.55rem;
    border-top: 1px solid var(--line);
  }
  .dev-head {
    display: block;
    font-size: 0.68rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.35rem;
  }

  .pip.ok {
    background: var(--ok);
  }
  .pip.warn {
    background: var(--warn);
  }
  .pip.bad {
    background: var(--bad);
  }
  .pip.idle {
    background: var(--muted);
  }

  .more {
    margin-top: 0.6rem;
    padding-top: 0.55rem;
    border-top: 1px solid var(--line);
    font-size: 0.8rem;
  }
  .more summary {
    cursor: pointer;
    color: var(--muted);
    font-size: 0.68rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    list-style: none;
  }
  .more summary::-webkit-details-marker {
    display: none;
  }
  .more summary::before {
    content: "▸ ";
  }
  .more[open] summary::before {
    content: "▾ ";
  }
  .more dl {
    margin: 0.45rem 0 0;
    display: grid;
    gap: 0.25rem;
  }
  .more dl div {
    display: flex;
    gap: 0.6rem;
  }
  .more dt {
    flex: none;
    width: 7.5rem;
    color: var(--muted);
  }
  .more dd {
    margin: 0;
  }

  .eng {
    margin-top: 0.6rem;
    background: none;
    border: 0;
    padding: 0.2rem 0;
    color: var(--muted);
    font-size: 0.74rem;
    cursor: pointer;
  }
  .eng:hover {
    color: var(--fg);
  }
</style>
