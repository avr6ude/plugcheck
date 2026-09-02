<script lang="ts">
  import type { Port, PortVerdict, CardStatus } from "./snapshot.svelte";
  import { renameCable } from "./snapshot.svelte";
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

  let editingName = $state(false);
  let nameDraft = $state("");
  function startRename() {
    nameDraft = port.history?.name ?? "";
    editingName = true;
  }
  async function commitRename() {
    editingName = false;
    if (port.history_sig) await renameCable(port.history_sig, nameDraft || null);
  }

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

  function speedName(s: string): string {
    return (
      {
        none: "—",
        usb2: "USB 2.0",
        usb3_gen1: "USB 3.2 Gen 1",
        usb3_gen2: "USB 3.2 Gen 2",
        usb4_gen3: "USB4 Gen 3",
        usb4_gen4: "USB4",
        thunderbolt3: "Thunderbolt 3",
        thunderbolt4: "Thunderbolt 4",
        displayport: "DisplayPort",
      }[s] ?? s
    );
  }
  function fastestDevice(): string {
    let best = "none";
    const rank = ["none", "usb2", "usb3_gen1", "usb3_gen2", "usb4_gen3", "thunderbolt3", "usb4_gen4", "thunderbolt4"];
    const walk = (ns: Port["devices"]) => {
      for (const d of ns) {
        if (rank.indexOf(d.speed) > rank.indexOf(best)) best = d.speed;
        walk(d.children);
      }
    };
    walk(port.devices);
    return best;
  }

  // key/value rows for the collapsible "Port details"
  const details = $derived(
    (
      [
        ["Port can carry", port.supported.length ? port.supported.join(", ") : null],
        [
          "Cable can carry",
          port.emarker.present
            ? speedName(port.emarker.max_speed)
            : "unknown (no e-marker)",
        ],
        ["Negotiated", port.provisioned.length ? port.provisioned.join(", ") : null],
        [
          "Fastest device wants",
          fastestDevice() !== "none" ? speedName(fastestDevice()) : null,
        ],
        ["Cable type", cableKind],
        ["Orientation", port.orientation != null ? `Position ${port.orientation}` : null],
        ["Display hot-plug", port.hpd ? "Detected" : null],
        port.display?.connection ? ["Display link", port.display.connection] : null,
        port.display?.depth ? ["Colour depth", port.display.depth] : null,
        port.display?.hdr ? ["HDR", "Yes"] : null,
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
      ].filter(Boolean) as [string, string | null][]
    ).filter(([, v]) => v != null) as [string, string][],
  );
</script>

<section class="port">
  <header>
    <span class="pip {worst}"></span>
    <h2>
      {#if port.history?.name && !editingName}{port.history.name}{:else}{verdict?.headline ??
          (port.occupied ? "Connected" : "Empty")}{/if}
    </h2>
    <span class="meta">{port.kind}</span>
  </header>

  {#if port.history}
    <div class="hist">
      {#if editingName}
        <input
          bind:value={nameDraft}
          placeholder="Name this cable / dock"
          onkeydown={(e) => e.key === "Enter" && commitRename()}
          onblur={commitRename}
        />
      {:else}
        <span>
          Seen {port.history.count}× · first {port.history.first_seen}
        </span>
        <button onclick={startRename}>
          {port.history.name ? "rename" : "name it"}
        </button>
      {/if}
    </div>
  {/if}

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

  {#if details.length || (port.charger?.pdos.length ?? 0) > 0}
    <details class="more">
      <summary>Port details</summary>
      <dl>
        {#each details as [k, v]}
          <div><dt>{k}</dt><dd>{v}</dd></div>
        {/each}
      </dl>
      {#if port.charger && port.charger.pdos.length}
        <div class="pdos">
          <span class="pdo-head">Power delivery contract</span>
          <table>
            <tbody>
              {#each port.charger.pdos as pdo}
                <tr
                  class:active={port.charger.negotiated_volts === pdo.volts}
                >
                  <td>{pdo.volts} V</td>
                  <td>{pdo.amps.toFixed(2)} A</td>
                  <td>{pdo.watts} W</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
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

  .hist {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: -0.3rem 0 0.5rem;
    font-size: 0.75rem;
    color: var(--muted);
  }
  .hist button {
    background: none;
    border: 1px solid var(--line);
    border-radius: 5px;
    padding: 0.05rem 0.4rem;
    color: var(--muted);
    font-size: 0.7rem;
    cursor: pointer;
  }
  .hist input {
    font-size: 0.78rem;
    padding: 0.2rem 0.4rem;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: var(--bg);
    color: var(--fg);
    width: 15rem;
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
    width: 9.5rem;
    color: var(--muted);
  }
  .more dd {
    margin: 0;
  }
  .pdos {
    margin-top: 0.6rem;
  }
  .pdo-head {
    display: block;
    font-size: 0.68rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.3rem;
  }
  .pdos table {
    border-collapse: collapse;
    font-variant-numeric: tabular-nums;
    font-size: 0.8rem;
  }
  .pdos td {
    padding: 0.12rem 0.9rem 0.12rem 0;
    color: var(--muted);
  }
  .pdos tr.active td {
    color: var(--fg);
    font-weight: 600;
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
