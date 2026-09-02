<script lang="ts">
  import type { Port, PortVerdict, CardStatus } from "./snapshot.svelte";
  import { renameCable } from "./snapshot.svelte";
  import DeviceTree from "./DeviceTree.svelte";

  let {
    port,
    verdict,
    technical,
    onEngineer,
  }: {
    port: Port;
    verdict: PortVerdict | undefined;
    technical: boolean;
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

  let editing = $state(false);
  let draft = $state("");
  let expandAll = $state(false);
  function startEdit() {
    draft = port.history?.name ?? "";
    editing = true;
  }
  async function commit() {
    editing = false;
    if (port.history_sig) await renameCable(port.history_sig, draft.trim() || null);
  }

  function count(): number {
    const w = (ns: Port["devices"]): number => ns.reduce((a, d) => a + 1 + w(d.children), 0);
    return w(port.devices);
  }
  function hubs(): number {
    const w = (ns: Port["devices"]): number =>
      ns.reduce((a, d) => a + (d.is_hub ? 1 : 0) + w(d.children), 0);
    return w(port.devices);
  }

  const conn = $derived(
    (
      [
        ["Connection active", port.occupied ? "Yes" : "No"],
        ["Active cable", port.cable_kind === "active" ? "Yes" : "No"],
        ["Optical cable", port.cable_kind === "optical" ? "Yes" : "No"],
        ["Plug orientation", port.orientation != null ? `Position ${port.orientation}` : null],
        ["Connections", port.connection_count != null ? `${port.connection_count}` : null],
        ["Plug events", port.plug_events != null ? `${port.plug_events}` : null],
        ["Overcurrent events", port.overcurrent_count != null ? `${port.overcurrent_count}` : null],
      ] as [string, string | null][]
    ).filter(([, v]) => v != null) as [string, string][],
  );
  const trans = $derived([
    ["Supported", port.supported.join(", ") || "—"],
    ["Negotiated", port.provisioned.join(", ") || "—"],
  ] as [string, string][]);
</script>

<section class="port">
  <div class="top">
    <span class="ico {worst}" aria-hidden="true">
      {#if worst === "ok"}
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M3.5 8.5 L6.5 11.5 L12.5 4.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>
      {:else if worst === "warn"}
        <svg viewBox="0 0 16 16" width="16" height="16"><path d="M8 2 L15 14 L1 14 Z" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linejoin="round"/><path d="M8 6.3 V9.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/><circle cx="8" cy="11.7" r="0.95" fill="currentColor"/></svg>
      {:else if worst === "bad"}
        <svg viewBox="0 0 16 16" width="16" height="16"><circle cx="8" cy="8" r="6.4" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="M5.6 5.6 L10.4 10.4 M10.4 5.6 L5.6 10.4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>
      {:else}
        <svg viewBox="0 0 16 16" width="16" height="16"><circle cx="8" cy="8" r="6.2" fill="none" stroke="currentColor" stroke-width="1.4"/></svg>
      {/if}
    </span>
    <div class="tt">
      <div class="pid">{port.id}</div>
      {#if editing}
        <input
          class="rename"
          bind:value={draft}
          placeholder="Name this cable / dock"
          onkeydown={(e) => e.key === "Enter" && commit()}
          onblur={commit}
        />
      {:else}
        <div class="nl">
          <button class="name" onclick={port.history ? startEdit : undefined}>{displayName}</button>
          {#if port.history}<span class="pencil" aria-hidden="true">✎</span>{/if}
        </div>
      {/if}
      {#if verdict?.subline}<div class="sl">{verdict.subline}</div>{/if}
    </div>
    {#if port.history && !editing}
      <button class="add" onclick={startEdit}>
        {port.history.name ? "Rename cable" : "Name this cable"} · seen {port.history.count}×
      </button>
    {/if}
  </div>

  {#if verdict?.cards.length}
    <div class="banners">
      {#each verdict.cards as c}
        <div class="banner {c.status}">
          <span class="bi" aria-hidden="true">
            {#if c.status === "ok"}✓{:else if c.status === "warn"}!{:else if c.status === "bad"}✕{:else}ⓘ{/if}
          </span>
          <div>
            <div class="bh">{c.head}</div>
            <div class="bt">{c.text}</div>
          </div>
        </div>
      {/each}
      {#each verdict.trust_flags as f}
        <div class="banner warn">
          <span class="bi">!</span>
          <div><div class="bh">Cable trust signal</div><div class="bt">{f}</div></div>
        </div>
      {/each}
    </div>
  {/if}

  {#if verdict?.cable_details.length}
    <div class="sect">
      <div class="sh">Cable details</div>
      <ul class="bul">
        {#each verdict.cable_details as b}<li>{b}</li>{/each}
      </ul>
    </div>
  {/if}

  {#if port.devices.length}
    <div class="sect">
      <div class="sh">
        Connected devices
        <span class="dim">· {count()}{hubs() ? ` · ${hubs()} hub${hubs() === 1 ? "" : "s"}` : ""}</span>
        {#if hubs()}
          <button class="lnk" onclick={() => (expandAll = !expandAll)}>{expandAll ? "Hide hubs" : "Show all"}</button>
        {/if}
      </div>
      <DeviceTree nodes={port.devices} forceOpen={expandAll} />
    </div>
  {/if}

  {#if port.charger && port.charger.pdos.length}
    <div class="sect">
      <div class="sh">USB-PD profiles</div>
      <table class="pdo">
        <tbody>
          {#each port.charger.pdos as pdo}
            <tr class:on={port.charger.negotiated_volts === pdo.volts}>
              <td><span class="pd" class:pdon={port.charger.negotiated_volts === pdo.volts}></span></td>
              <td>{pdo.volts} V @ {pdo.amps.toFixed(2)} A</td>
              <td>{pdo.watts} W</td>
              <td>{port.charger.negotiated_volts === pdo.volts ? "active" : ""}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  {#if technical}
    <div class="tech">
      <div class="tg">
        <div class="sh">Connection</div>
        <dl>{#each conn as [k, v]}<div><dt>{k}</dt><dd>{v}</dd></div>{/each}</dl>
      </div>
      <div class="tg">
        <div class="sh">Transports</div>
        <dl>{#each trans as [k, v]}<div><dt>{k}</dt><dd>{v}</dd></div>{/each}</dl>
      </div>
      <button class="lnk raw" onclick={() => onEngineer(port.id)}>
        All raw IOKit properties ({Object.keys(port.raw).length}) →
      </button>
    </div>
  {/if}
</section>

<style>
  .port {
    background: var(--card);
    border-radius: 10px;
    border: 0.5px solid var(--line);
    padding: 0.8rem 0.9rem;
  }

  .top {
    display: flex;
    gap: 0.55rem;
    align-items: flex-start;
  }
  .ico {
    flex: none;
    margin-top: 0.15rem;
  }
  .ico.ok {
    color: var(--ok);
  }
  .ico.warn {
    color: var(--warn);
  }
  .ico.bad {
    color: var(--bad);
  }
  .ico.idle {
    color: var(--muted);
  }
  .tt {
    min-width: 0;
    flex: 1;
  }
  .pid {
    font: 0.68rem/1.2 var(--mono);
    color: var(--muted);
  }
  .nl {
    display: flex;
    align-items: baseline;
    gap: 0.3rem;
  }
  .name {
    font: 650 1rem/1.3 inherit;
    color: var(--fg);
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
  }
  .name:hover {
    color: var(--accent);
  }
  .pencil {
    color: var(--muted);
    font-size: 0.72rem;
    opacity: 0;
  }
  .top:hover .pencil {
    opacity: 1;
  }
  .rename {
    font: 650 1rem/1.3 inherit;
    color: var(--fg);
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: 5px;
    padding: 0.05rem 0.35rem;
    width: 100%;
  }
  .sl {
    color: var(--muted);
    font-size: 0.82rem;
    margin-top: 0.05rem;
  }
  .add {
    flex: none;
    background: none;
    border: 0;
    padding: 0;
    color: var(--accent);
    font-size: 0.74rem;
    cursor: pointer;
    white-space: nowrap;
  }

  /* verdict banners */
  .banners {
    display: grid;
    gap: 0.35rem;
    margin: 0.7rem 0 0.2rem;
  }
  .banner {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem 0.65rem;
    border-radius: 8px;
    border: 0.5px solid var(--line);
    background: color-mix(in srgb, var(--muted) 8%, var(--card));
  }
  .banner.ok {
    border-color: color-mix(in srgb, var(--ok) 40%, transparent);
    background: color-mix(in srgb, var(--ok) 11%, var(--card));
  }
  .banner.warn {
    border-color: color-mix(in srgb, var(--warn) 45%, transparent);
    background: color-mix(in srgb, var(--warn) 12%, var(--card));
  }
  .banner.bad {
    border-color: color-mix(in srgb, var(--bad) 45%, transparent);
    background: color-mix(in srgb, var(--bad) 11%, var(--card));
  }
  .bi {
    flex: none;
    width: 1rem;
    text-align: center;
    font-weight: 700;
    font-size: 0.85rem;
  }
  .banner.ok .bi {
    color: var(--ok);
  }
  .banner.warn .bi {
    color: var(--warn);
  }
  .banner.bad .bi {
    color: var(--bad);
  }
  .banner.idle .bi {
    color: var(--muted);
  }
  .bh {
    font-weight: 650;
    font-size: 0.86rem;
    color: var(--fg);
  }
  .banner.ok .bh {
    color: color-mix(in srgb, var(--ok) 75%, var(--fg));
  }
  .bt {
    font-size: 0.8rem;
    color: var(--muted);
    line-height: 1.4;
    margin-top: 0.1rem;
  }

  /* sections */
  .sect {
    margin-top: 0.75rem;
  }
  .sh {
    font-size: 0.66rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.3rem;
  }
  .sh .dim {
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
  }
  .lnk {
    background: none;
    border: 0;
    padding: 0;
    margin-left: 0.5rem;
    color: var(--accent);
    font-size: 0.72rem;
    cursor: pointer;
  }
  .bul {
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 0.83rem;
    line-height: 1.55;
  }
  .bul li {
    padding-left: 0.9rem;
    position: relative;
  }
  .bul li::before {
    content: "·";
    position: absolute;
    left: 0.2rem;
    color: var(--muted);
  }

  .pdo {
    border-collapse: collapse;
    font-size: 0.82rem;
  }
  .pdo td {
    padding: 0.1rem 0.7rem 0.1rem 0;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }
  .pdo tr.on td {
    color: var(--fg);
  }
  .pd {
    display: inline-block;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    border: 1px solid var(--line);
  }
  .pd.pdon {
    background: var(--ok);
    border-color: var(--ok);
  }
  .pdo td:last-child {
    color: var(--ok);
    font-size: 0.72rem;
  }

  .tech {
    margin-top: 0.8rem;
    padding-top: 0.6rem;
    border-top: 0.5px solid var(--line);
  }
  .tg {
    margin-bottom: 0.5rem;
  }
  .tech dl {
    margin: 0;
    display: grid;
    gap: 0.08rem;
  }
  .tech dl > div {
    display: flex;
    gap: 0.6rem;
    font-size: 0.8rem;
  }
  .tech dt {
    flex: none;
    width: 10rem;
    color: var(--muted);
  }
  .tech dd {
    margin: 0;
    font-family: var(--mono);
    font-size: 0.76rem;
  }
  .raw {
    margin-left: 0;
    margin-top: 0.3rem;
  }
</style>
