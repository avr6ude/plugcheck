{#snippet symbol(status: CardStatus, size: number)}
  <svg class="symbol {status}" viewBox="0 0 20 20" width={size} height={size} aria-hidden="true">
    <circle cx="10" cy="10" r="10" />
    <g fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      {#if status === "ok"}<path d="M6 10.4l2.7 2.7L14 7.3" />
      {:else if status === "warn"}<path d="M10 5.6v5.3M10 14.3v.1" />
      {:else if status === "bad"}<path d="M7 7l6 6M13 7l-6 6" />
      {:else}<path d="M6.5 10h7" />{/if}
    </g>
  </svg>
{/snippet}

<script lang="ts">
  import { createTabs, melt } from "@melt-ui/svelte";
  import type { Port, PortVerdict, CardStatus } from "./snapshot.svelte";
  import DeviceTree from "./DeviceTree.svelte";
  import { renameCable } from "./snapshot.svelte";
  import { portLabel } from "./MacScene.svelte";

  let { port, verdict, technical }: { port: Port; verdict: PortVerdict | undefined; technical: boolean } = $props();
  let tab = $state("overview");
  const { elements: { root, list, trigger } } = createTabs({
    defaultValue: "overview",
    onValueChange: ({ next }) => { tab = next; return next; },
  });
  const shown = $derived(tab === "technical" && !technical ? "overview" : tab); // technical tab vanished while open
  // The data verdict is the answer; other cards are supporting findings and never colour the headline.
  const dataCard = $derived(verdict?.cards.find((c) => c.kind === "data"));
  const findings = $derived((verdict?.cards ?? []).filter((c) => c !== dataCard));
  const status = $derived<CardStatus>(dataCard?.status ?? (port.occupied ? "ok" : "idle"));
  const device = $derived(port.devices[0]);
  const deviceCount = $derived(port.devices.reduce((n, d) => n + 1 + d.children.length, 0));
  const link = $derived(port.provisioned[0] ?? (port.active_transport && port.active_transport !== "none" ? port.active_transport : "None"));
  const charger = $derived(port.charger);
  // `supported` is unordered and mixes in DisplayPort, which says nothing about data speed.
  const DATA = ["USB 2.0", "USB 3.2", "Thunderbolt / USB4"];
  const portMax = $derived(DATA.findLast((t) => port.supported.includes(t)) ?? "Unknown");
  // Backend notes repeat Link/Device; fold their extra detail into those rows and keep the rest as footnotes.
  const linkRate = $derived(verdict?.cable_details.find((d) => d.startsWith("Link:"))?.match(/\(([^)]+)\)/)?.[1]);
  const notes = $derived((verdict?.cable_details ?? []).filter((d) => !/^(Link|Device):/.test(d)));
  const sentence = $derived(port.occupied
    ? `${device?.name ?? verdict?.headline ?? "A device"} is connected at ${link}.`
    : `Nothing is connected to ${portLabel(port)}.`);
  const answer = $derived(port.occupied
    ? (dataCard ? `${capital(dataCard.head.replace(/^.*? — /, ""))}. ${dataCard.text}` : verdict?.subline ?? "")
    : `This port supports up to ${portMax}.`);
  const cable = $derived(port.cable_kind && port.cable_kind !== "unknown" ? port.cable_kind[0].toUpperCase() + port.cable_kind.slice(1) : "Unknown");

  // The sentence above already names the link, so the subline starts at the assessment.
  function capital(t: string) { return t.charAt(0).toUpperCase() + t.slice(1); }

  function rename(e: Event) {
    const name = (e.currentTarget as HTMLInputElement).value.trim();
    if (port.history_sig) renameCable(port.history_sig, name || null);
  }
</script>

<section class="inspector" use:melt={$root} aria-label="Port details">
  <header class="head">
    {@render symbol(status, 24)}
    <div>
      <h2>{sentence}</h2>
      {#if answer}<p>{answer}</p>{/if}
    </div>
  </header>

  {#if port.occupied || technical}
    <div class="segmented" use:melt={$list} aria-label="Port detail">
      <button class:on={shown === "overview"} use:melt={$trigger("overview")}>Overview</button>
      {#if port.occupied}
        <button class:on={shown === "devices"} use:melt={$trigger("devices")}>Devices{deviceCount ? ` (${deviceCount})` : ""}</button>
        <button class:on={shown === "power"} use:melt={$trigger("power")}>Power</button>
      {/if}
      {#if technical}
        <button class:on={shown === "technical"} use:melt={$trigger("technical")}>Technical</button>
      {/if}
    </div>
  {/if}

  {#if shown === "overview" && !port.occupied}
    <!-- the header already says everything an empty port can -->
  {:else if shown === "overview"}
    {#if findings.length}
      <ul class="group findings">
        {#each findings as card}
          <li class="row">{@render symbol(card.status, 16)}<span><b>{card.head}</b><small>{card.text}</small></span></li>
        {/each}
      </ul>
    {/if}

    <h3 class="list-title">Connection</h3>
    <dl class="group">
      <div class="row"><dt>Link</dt><dd>{link}{linkRate ? ` · ${linkRate}` : ""}</dd></div>
      <div class="row"><dt>Port supports</dt><dd>{portMax}</dd></div>
      <div class="row"><dt>Device</dt><dd>{device ? `${device.name}${device.vendor ? ` · ${device.vendor}` : ""}` : "None"}</dd></div>
      <div class="row"><dt>Cable</dt><dd>{cable}</dd></div>
      <div class="row"><dt>E-marker</dt><dd>{port.emarker.present ? port.emarker.max_speed : "Not readable"}</dd></div>
      <div class="row"><dt>Charger</dt><dd>{charger?.watts != null ? `${charger.watts} W` : "None"}</dd></div>
    </dl>

    {#each notes as note}<p class="group-note">{note}</p>{/each}

    {#if port.history}
      <h3 class="list-title">This Cable</h3>
      <dl class="group">
        <div class="row"><dt><label for="cable-name">Name</label></dt><dd><input id="cable-name" class="field" placeholder="Unnamed" value={port.history.name ?? ""} onchange={rename} onkeydown={(e) => e.key === "Enter" && e.currentTarget.blur()} /></dd></div>
        <div class="row"><dt>Seen</dt><dd>{port.history.count} {port.history.count === 1 ? "time" : "times"}</dd></div>
      </dl>
    {/if}
  {:else if shown === "devices"}
    {#if port.devices.length}
      <div class="group tree"><DeviceTree nodes={port.devices} forceOpen={true} /></div>
    {:else}<p class="empty-note">No devices are connected to this port.</p>{/if}
  {:else if shown === "power"}
    {#if charger}
      <dl class="group">
        <div class="row"><dt>Drawing now</dt><dd>{charger.live_watts ?? charger.watts ?? 0} W</dd></div>
        <div class="row"><dt>Battery</dt><dd>{charger.fully_charged ? "Full" : charger.is_charging ? "Charging" : "Not charging"}{charger.battery_percent != null ? ` · ${charger.battery_percent}%` : ""}</dd></div>
        <div class="row"><dt>Contract</dt><dd>{charger.negotiated_volts != null && charger.negotiated_amps != null ? `${charger.negotiated_volts} V · ${charger.negotiated_amps.toFixed(2)} A` : "None"}</dd></div>
      </dl>
      {#if charger.pdos.length}
        <h3 class="list-title">Charger Offers</h3>
        <dl class="group">
          {#each charger.pdos as pdo}<div class="row"><dt>{pdo.volts} V · {pdo.amps.toFixed(2)} A</dt><dd>{pdo.watts} W</dd></div>{/each}
        </dl>
      {/if}
    {:else}<p class="empty-note">This port isn’t supplying or receiving power.</p>{/if}
  {:else}
    <dl class="group raw">
      <div class="row"><dt>Port ID</dt><dd>{port.id}</dd></div>
      {#each Object.entries(port.raw) as [k, v]}<div class="row"><dt>{k}</dt><dd>{v}</dd></div>{/each}
    </dl>
  {/if}
</section>

<style>
  .head { display: flex; align-items: flex-start; gap: 10px; margin-bottom: 16px; }
  .head > .symbol { margin-top: 1px; }
  .head h2 { margin: 0; font-size: 17px; font-weight: 600; line-height: 1.25; }
  .head p { margin: 2px 0 0; color: var(--muted); }
  .symbol { flex: none; }
  .symbol circle { fill: var(--tertiary); }
  .symbol.ok circle { fill: var(--ok); } .symbol.warn circle { fill: var(--warn); } .symbol.bad circle { fill: var(--bad); }

  /* NSSegmentedControl */
  .segmented { display: flex; gap: 2px; padding: 2px; margin-bottom: 18px; border-radius: 7px; background: var(--fill); }
  .segmented button { flex: 1; height: 22px; border: 0; border-radius: 5px; background: none; font-size: 13px; white-space: nowrap; }
  .segmented button.on { background: var(--btn); box-shadow: var(--btn-edge); }


  .findings > .row { justify-content: flex-start; align-items: flex-start; gap: 9px; padding-block: 9px; }
  .findings .symbol { margin-top: 1px; }
  .findings span { display: grid; gap: 2px; }
  .findings b { font-weight: 400; }
  .findings small { color: var(--muted); font-size: 11px; line-height: 1.35; }

  .field { width: 12rem; max-width: 100%; height: 22px; padding: 0 6px; border: 0; border-radius: 5px; background: var(--fill); text-align: right; -webkit-user-select: text; user-select: text; }
  .field:focus { text-align: left; }
  .tree { padding: 6px 12px; }
  .raw > .row { font: 11px/1.35 var(--mono); }
  .raw dd { color: var(--fg); -webkit-user-select: text; user-select: text; }
</style>
