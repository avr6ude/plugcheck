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

  const glyph: Record<CardStatus, string> = { ok: "✓", warn: "!", bad: "✕", idle: "" };
  const rank: Record<CardStatus, number> = { idle: 0, ok: 1, warn: 2, bad: 3 };
  const worst = $derived(
    (verdict?.cards ?? []).reduce<CardStatus>(
      (w, c) => (rank[c.status] > rank[w] ? c.status : w),
      "ok",
    ),
  );

  const displayName = $derived(
    port.history?.name ?? verdict?.headline ?? (port.occupied ? "Connected" : "Empty"),
  );

  let editing = $state(false);
  let draft = $state("");
  function startEdit() {
    draft = port.history?.name ?? "";
    editing = true;
  }
  async function commit() {
    editing = false;
    if (port.history_sig) await renameCable(port.history_sig, draft.trim() || null);
  }

  function deviceCount(): number {
    const w = (ns: Port["devices"]): number =>
      ns.reduce((a, d) => a + 1 + w(d.children), 0);
    return w(port.devices);
  }

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
    const order = ["none", "usb2", "usb3_gen1", "usb3_gen2", "usb4_gen3", "thunderbolt3", "usb4_gen4", "thunderbolt4"];
    const walk = (ns: Port["devices"]) => {
      for (const d of ns) {
        if (order.indexOf(d.speed) > order.indexOf(best)) best = d.speed;
        walk(d.children);
      }
    };
    walk(port.devices);
    return best;
  }

  const cableKind = $derived(
    ({ passive: "Passive", active: "Active", optical: "Optical" } as Record<string, string>)[
      port.cable_kind
    ] ?? null,
  );

  const details = $derived(
    (
      [
        ["Port can carry", port.supported.length ? port.supported.join(", ") : null],
        [
          "Cable can carry",
          port.emarker.present ? speedName(port.emarker.max_speed) : "Unknown (no e-marker)",
        ],
        ["Negotiated", port.provisioned.length ? port.provisioned.join(", ") : null],
        [
          "Fastest device wants",
          fastestDevice() !== "none" ? speedName(fastestDevice()) : null,
        ],
        ["Cable type", cableKind],
        ["Orientation", port.orientation != null ? `Position ${port.orientation}` : null],
        ["Display hot-plug", port.hpd ? "Detected" : null],
        ["Connections", port.connection_count != null ? `${port.connection_count}` : null],
        ["Plug events", port.plug_events != null ? `${port.plug_events}` : null],
        [
          "Overcurrent",
          port.overcurrent_count != null ? `${port.overcurrent_count}` : null,
        ],
      ] as [string, string | null][]
    ).filter(([, v]) => v != null) as [string, string][],
  );
</script>

<section class="group">
  <!-- headline -->
  <div class="hd">
    <span class="pip {worst}"></span>
    {#if editing}
      <input
        class="rename"
        bind:value={draft}
        placeholder="Name this cable / dock"
        onkeydown={(e) => e.key === "Enter" && commit()}
        onblur={commit}
      />
    {:else}
      <button class="name" onclick={port.history ? startEdit : undefined} title="Rename">
        {displayName}
      </button>
      {#if port.history}<span class="edit" aria-hidden="true">✎</span>{/if}
    {/if}
  </div>

  {#if port.history && !editing}
    <div class="sub">Seen {port.history.count}× · since {port.history.first_seen}</div>
  {/if}

  <!-- verdict blocks -->
  {#each verdict?.cards ?? [] as card}
    <div class="block">
      <div class="block-hd">
        {#if glyph[card.status]}<span class="dot {card.status}">{glyph[card.status]}</span>{/if}
        <span>{card.title}</span>
      </div>
      {#if card.rows.length}
        <dl class="kv">
          {#each card.rows as [k, v]}
            <div><dt>{k}</dt><dd>{v}</dd></div>
          {/each}
        </dl>
      {/if}
      {#if card.text}<p class="line">{card.text}</p>{/if}
    </div>
  {/each}

  {#each verdict?.trust_flags ?? [] as flag}
    <div class="block warn">
      <div class="block-hd"><span class="dot warn">!</span><span>Trust</span></div>
      <p class="line">{flag}</p>
    </div>
  {/each}

  {#if port.devices.length}
    <details class="disc" open>
      <summary><span class="chev" aria-hidden="true"><svg viewBox="0 0 12 12" width="8" height="8"><path d="M4 2 L8 6 L4 10" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></span> Connected devices ({deviceCount()})</summary>
      <DeviceTree nodes={port.devices} />
    </details>
  {/if}

  {#if details.length || (port.charger?.pdos.length ?? 0) > 0}
    <details class="disc">
      <summary><span class="chev" aria-hidden="true"><svg viewBox="0 0 12 12" width="8" height="8"><path d="M4 2 L8 6 L4 10" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></span> Port details</summary>
      <dl class="kv wide">
        {#each details as [k, v]}
          <div><dt>{k}</dt><dd>{v}</dd></div>
        {/each}
      </dl>
      {#if port.charger && port.charger.pdos.length}
        <div class="pdo-wrap">
          <span class="pdo-h">Power-delivery contract</span>
          <table>
            <tbody>
              {#each port.charger.pdos as pdo}
                <tr class:active={port.charger.negotiated_volts === pdo.volts}>
                  <td>{pdo.volts} V</td><td>{pdo.amps.toFixed(2)} A</td><td>{pdo.watts} W</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </details>
  {/if}

  <button class="raw" onclick={() => onEngineer(port.id)}>
    <span class="chev" aria-hidden="true"><svg viewBox="0 0 12 12" width="8" height="8"><path d="M4 2 L8 6 L4 10" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></span> Raw IOKit data
  </button>
</section>

<style>
  .group {
    background: var(--card);
    border: 0.5px solid var(--line);
    border-radius: 12px;
    overflow: hidden;
  }

  .hd {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.7rem 0.9rem;
  }
  .pip {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
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
  .name {
    font: 600 0.98rem/1.2 inherit;
    color: var(--fg);
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    text-align: left;
  }
  .name:hover {
    text-decoration: underline dotted;
  }
  .edit {
    color: var(--muted);
    font-size: 0.75rem;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .hd:hover .edit {
    opacity: 1;
  }
  .rename {
    font: 600 0.98rem/1.2 inherit;
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: 6px;
    padding: 0.15rem 0.4rem;
    flex: 1;
    min-width: 0;
  }
  .sub {
    padding: 0 0.9rem 0.6rem;
    margin-top: -0.35rem;
    font-size: 0.74rem;
    color: var(--muted);
  }

  .block {
    padding: 0.6rem 0.9rem;
    border-top: 0.5px solid var(--line);
  }
  .block-hd {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.66rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.35rem;
  }
  .dot {
    width: 15px;
    height: 15px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 0.62rem;
    font-weight: 700;
    color: #fff;
  }
  .dot.ok {
    background: var(--ok);
  }
  .dot.warn {
    background: var(--warn);
  }
  .dot.bad {
    background: var(--bad);
  }
  .line {
    margin: 0;
    font-size: 0.85rem;
    line-height: 1.45;
  }
  .block.warn .line {
    color: var(--warn-fg);
  }

  dl.kv {
    margin: 0;
    display: grid;
    gap: 0.15rem;
  }
  dl.kv > div {
    display: flex;
    gap: 0.75rem;
    font-size: 0.84rem;
  }
  dl.kv dt {
    flex: none;
    width: 6.5rem;
    color: var(--muted);
  }
  dl.kv.wide dt {
    width: 10rem;
  }
  dl.kv dd {
    margin: 0;
  }

  .disc {
    border-top: 0.5px solid var(--line);
  }
  .disc summary,
  .raw {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    width: 100%;
    padding: 0.55rem 0.9rem;
    font-size: 0.74rem;
    color: var(--muted);
    cursor: pointer;
    list-style: none;
    background: none;
    border: 0;
    text-align: left;
  }
  .raw {
    border-top: 0.5px solid var(--line);
  }
  .disc summary::-webkit-details-marker {
    display: none;
  }
  .disc summary:hover,
  .raw:hover {
    color: var(--fg);
  }
  .chev {
    flex: none;
    width: 12px;
    height: 12px;
    display: grid;
    place-items: center;
    color: var(--muted);
    transition: transform 0.15s ease;
  }
  .disc[open] summary .chev {
    transform: rotate(90deg);
  }
  .disc :global(.tree),
  .disc dl.kv,
  .disc .pdo-wrap {
    padding: 0 0.9rem 0.7rem 1.9rem;
  }

  .pdo-wrap {
    margin-top: 0.5rem;
  }
  .pdo-h {
    display: block;
    font-size: 0.64rem;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.25rem;
  }
  .pdo-wrap table {
    border-collapse: collapse;
    font-variant-numeric: tabular-nums;
    font-size: 0.8rem;
  }
  .pdo-wrap td {
    padding: 0.1rem 0.9rem 0.1rem 0;
    color: var(--muted);
  }
  .pdo-wrap tr.active td {
    color: var(--fg);
    font-weight: 600;
  }
</style>
