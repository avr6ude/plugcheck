<script lang="ts">
  import type { Port, PortVerdict, CardStatus, VerdictCard } from "./snapshot.svelte";
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

  // chip strip: data / power / video at a glance
  const chips = $derived(
    (verdict?.cards ?? []).filter((c) => c.chip) as VerdictCard[],
  );
  const chipLabel: Record<string, string> = {
    data: "Data",
    charging: "Power",
    display: "Video",
  };

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
  function hubCount(): number {
    const w = (ns: Port["devices"]): number =>
      ns.reduce((a, d) => a + (d.is_hub ? 1 : 0) + w(d.children), 0);
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
        ["Port supports", port.supported.length ? port.supported.join(", ") : null],
        [
          "Cable supports",
          port.emarker.present ? speedName(port.emarker.max_speed) : "Unknown (no e-marker)",
        ],
        ["Negotiated", port.provisioned.length ? port.provisioned.join(", ") : null],
        [
          "Fastest device",
          fastestDevice() !== "none" ? speedName(fastestDevice()) : null,
        ],
        ["Cable type", cableKind],
        ["Plug orientation", port.orientation != null ? `Position ${port.orientation}` : null],
        ["Connections", port.connection_count != null ? `${port.connection_count}` : null],
        ["Plug events", port.plug_events != null ? `${port.plug_events}` : null],
        ["Overcurrent events", port.overcurrent_count != null ? `${port.overcurrent_count}` : null],
      ] as [string, string | null][]
    ).filter(([, v]) => v != null) as [string, string][],
  );
</script>

<section class="port">
  <div class="hdr">
    <span class="dot {worst}"></span>
    {#if editing}
      <input
        class="rename"
        bind:value={draft}
        placeholder="Name this cable / dock"
        onkeydown={(e) => e.key === "Enter" && commit()}
        onblur={commit}
      />
    {:else}
      <button class="title" onclick={port.history ? startEdit : undefined}>{displayName}</button>
      {#if port.history}<span class="pencil" aria-hidden="true">✎</span>{/if}
    {/if}
    {#if port.history && !editing}
      <span class="seen">seen {port.history.count}× · {port.history.first_seen}</span>
    {/if}
  </div>

  {#if chips.length}
    <div class="chips">
      {#each chips as c}
        <span class="chip {c.status}">
          <b>{chipLabel[c.kind] ?? c.title}</b>
          {c.chip}
        </span>
      {/each}
    </div>
  {/if}

  <div class="fields">
    {#each verdict?.cards ?? [] as card}
      <div class="field {card.status}">
        <div class="k">{card.title}</div>
        <div class="v">
          {#if card.rows.length}
            {#each card.rows as [k, v]}
              <div class="sub"><span>{k}</span>{v}</div>
            {/each}
          {/if}
          {#if card.text}<div>{card.text}</div>{/if}
        </div>
      </div>
    {/each}

    {#each verdict?.trust_flags ?? [] as flag}
      <div class="field warn">
        <div class="k">Trust</div>
        <div class="v">{flag}</div>
      </div>
    {/each}

    {#if port.devices.length}
      <div class="field">
        <div class="k">
          Devices
          <span class="cnt">{deviceCount()}{hubCount() ? ` · ${hubCount()} hub${hubCount() === 1 ? "" : "s"}` : ""}</span>
        </div>
        <div class="v">
          <DeviceTree nodes={port.devices} />
        </div>
      </div>
    {/if}
  </div>

  {#if details.length || (port.charger?.pdos.length ?? 0) > 0}
    <details class="disc">
      <summary>
        <svg class="chev" viewBox="0 0 12 12" width="9" height="9" aria-hidden="true"
          ><path d="M4 2 L8 6 L4 10" fill="none" stroke="currentColor" stroke-width="1.7"
            stroke-linecap="round" stroke-linejoin="round" /></svg
        >
        Details
      </summary>
      <div class="disc-body">
        <dl>
          {#each details as [k, v]}
            <div><dt>{k}</dt><dd>{v}</dd></div>
          {/each}
        </dl>
        {#if port.charger && port.charger.pdos.length}
          <div class="pdo">
            <span class="pdo-h">Power-delivery contract</span>
            <table>
              <tbody>
                {#each port.charger.pdos as pdo}
                  <tr class:on={port.charger.negotiated_volts === pdo.volts}>
                    <td>{pdo.volts} V</td><td>{pdo.amps.toFixed(2)} A</td><td>{pdo.watts} W</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
        <button class="raw" onclick={() => onEngineer(port.id)}>Raw IOKit data →</button>
      </div>
    </details>
  {/if}
</section>

<style>
  .port {
    background: var(--card);
    border-radius: 10px;
    border: 0.5px solid var(--line);
    padding: 0.75rem 0.85rem 0.6rem;
  }

  /* header */
  .hdr {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
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
  .dot.idle {
    background: var(--muted);
  }
  .title {
    font: 600 0.95rem/1.2 inherit;
    color: var(--fg);
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
  }
  .title:hover {
    color: var(--accent);
  }
  .pencil {
    color: var(--muted);
    font-size: 0.72rem;
    opacity: 0;
    transition: opacity 0.1s;
  }
  .hdr:hover .pencil {
    opacity: 1;
  }
  .rename {
    font: 600 0.95rem/1.2 inherit;
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: 5px;
    padding: 0.1rem 0.35rem;
    flex: 1;
    min-width: 0;
  }
  .seen {
    margin-left: auto;
    color: var(--muted);
    font-size: 0.7rem;
    white-space: nowrap;
  }

  /* chip strip */
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin: 0.55rem 0 0.15rem;
  }
  .chip {
    font-size: 0.74rem;
    padding: 0.12rem 0.45rem;
    border-radius: 999px;
    border: 1px solid var(--line);
    color: var(--fg);
    background: color-mix(in srgb, var(--card) 88%, var(--muted));
  }
  .chip b {
    font-weight: 600;
    color: var(--muted);
    margin-right: 0.25rem;
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .chip.ok {
    border-color: color-mix(in srgb, var(--ok) 45%, transparent);
    background: color-mix(in srgb, var(--ok) 12%, var(--card));
  }
  .chip.warn {
    border-color: color-mix(in srgb, var(--warn) 50%, transparent);
    background: color-mix(in srgb, var(--warn) 14%, var(--card));
  }
  .chip.bad {
    border-color: color-mix(in srgb, var(--bad) 50%, transparent);
    background: color-mix(in srgb, var(--bad) 12%, var(--card));
  }

  /* get-info style fields */
  .fields {
    margin-top: 0.5rem;
    display: grid;
    gap: 0.02rem;
  }
  .field {
    display: grid;
    grid-template-columns: 6.5rem 1fr;
    gap: 0.6rem;
    padding: 0.32rem 0;
    border-top: 0.5px solid var(--line);
    font-size: 0.84rem;
    align-items: start;
  }
  .field:first-child {
    border-top: 0;
  }
  .k {
    color: var(--muted);
    display: flex;
    flex-direction: column;
  }
  .cnt {
    font-size: 0.72rem;
  }
  .v {
    min-width: 0;
    line-height: 1.4;
  }
  .v .sub {
    display: flex;
    gap: 0.5rem;
  }
  .v .sub span {
    color: var(--muted);
    min-width: 4rem;
  }
  .field.warn .v {
    color: var(--warn-fg);
  }
  .field.bad .v {
    color: var(--bad);
  }

  /* details disclosure */
  .disc {
    margin-top: 0.4rem;
  }
  .disc summary {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0 0.2rem;
    font-size: 0.76rem;
    color: var(--muted);
    cursor: pointer;
    list-style: none;
  }
  .disc summary::-webkit-details-marker {
    display: none;
  }
  .disc summary:hover {
    color: var(--fg);
  }
  .chev {
    transition: transform 0.14s ease;
  }
  .disc[open] summary .chev {
    transform: rotate(90deg);
  }
  .disc-body {
    padding: 0.3rem 0 0.2rem 0.2rem;
  }
  .disc dl {
    margin: 0;
    display: grid;
    gap: 0.12rem;
  }
  .disc dl > div {
    display: flex;
    gap: 0.6rem;
    font-size: 0.8rem;
  }
  .disc dt {
    flex: none;
    width: 9rem;
    color: var(--muted);
  }
  .disc dd {
    margin: 0;
  }
  .pdo {
    margin-top: 0.5rem;
  }
  .pdo-h {
    display: block;
    font-size: 0.66rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }
  .pdo table {
    border-collapse: collapse;
    font: 0.78rem/1.5 var(--mono);
  }
  .pdo td {
    padding: 0.06rem 0.8rem 0.06rem 0;
    color: var(--muted);
  }
  .pdo tr.on td {
    color: var(--fg);
    font-weight: 600;
  }
  .raw {
    margin-top: 0.55rem;
    background: none;
    border: 0;
    padding: 0;
    color: var(--accent);
    font-size: 0.76rem;
    cursor: pointer;
  }
</style>
