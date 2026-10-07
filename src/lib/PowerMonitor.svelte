<script lang="ts">
  import { onMount } from "svelte";
  import { store, refresh } from "./snapshot.svelte";

  const charger = $derived(
    store.snapshot?.ports.map((p) => p.charger).find((c) => c) ?? null,
  );
  const pts = $derived(store.power);
  const cur = $derived(pts.length ? pts[pts.length - 1].w : 0);
  const peak = $derived(pts.reduce((m, p) => Math.max(m, p.w), 0));
  const ceiling = $derived(charger?.watts ?? 0);
  const maxY = $derived(Math.max(20, ceiling, peak) * 1.1);
  const W = 560;
  const H = 130;

  const line = $derived(() => {
    if (pts.length < 2) return "";
    const n = pts.length;
    return pts
      .map(
        (p, i) =>
          `${i === 0 ? "M" : "L"} ${((i / (n - 1)) * W).toFixed(1)} ${(
            H -
            (p.w / maxY) * H
          ).toFixed(1)}`,
      )
      .join(" ");
  });
  const area = $derived(() => {
    const l = line();
    return l ? `${l} L ${W} ${H} L 0 ${H} Z` : "";
  });
  const ceilY = $derived(ceiling ? H - (ceiling / maxY) * H : -1);

  // faster refresh while this view is open
  onMount(() => {
    const id = setInterval(refresh, 1500);
    return () => clearInterval(id);
  });
</script>

<p class="big"><span class="n">{cur.toFixed(0)}</span> W{#if ceiling}<span class="of">from a {ceiling} W adapter</span>{/if}</p>

<div class="group chart-box">
  <svg viewBox="0 0 {W} {H}" preserveAspectRatio="none" class="chart" aria-label="Power draw over time">
    {#if ceilY >= 0}<line x1="0" y1={ceilY} x2={W} y2={ceilY} class="ceil" />{/if}
    <path d={area()} class="fill" />
    <path d={line()} class="stroke" />
  </svg>
</div>
<p class="group-note">Peak {peak.toFixed(0)} W · last {pts.length} readings</p>

<h3 class="list-title">Charger</h3>
<dl class="group">
  {#if charger}
    <div class="row"><dt>Adapter</dt><dd>{charger.watts ? `${charger.watts} W` : "Unknown"}</dd></div>
    <div class="row"><dt>Contract</dt><dd>{charger.negotiated_volts != null && charger.negotiated_amps != null ? `${charger.negotiated_volts.toFixed(0)} V · ${charger.negotiated_amps.toFixed(2)} A` : "None"}</dd></div>
    <div class="row"><dt>Into battery</dt><dd>{charger.live_watts != null ? `${charger.live_watts.toFixed(0)} W` : "Unknown"}</dd></div>
    <div class="row"><dt>Battery</dt><dd>{charger.battery_percent != null ? `${charger.battery_percent}%` : "Unknown"}{#if charger.minutes_to_full} · full in {Math.floor(charger.minutes_to_full / 60)} h {charger.minutes_to_full % 60} min{/if}</dd></div>
    <div class="row"><dt>State</dt><dd>{charger.fully_charged ? "Full" : charger.is_charging ? "Charging" : "Not charging"}</dd></div>
  {:else}
    <div class="row"><dt>Adapter</dt><dd>None connected</dd></div>
  {/if}
</dl>

<style>
  .big { margin: 0 0 12px 12px; color: var(--muted); font-size: 15px; }
  .n { color: var(--fg); font-size: 34px; font-weight: 600; font-variant-numeric: tabular-nums; letter-spacing: -.02em; }
  .of { margin-left: 8px; }
  .chart-box { padding: 10px; }
  .chart { display: block; width: 100%; height: 140px; }
  .stroke { fill: none; stroke: var(--accent); stroke-width: 1.5; vector-effect: non-scaling-stroke; }
  .fill { fill: color-mix(in srgb, var(--accent) 12%, transparent); }
  .ceil { stroke: var(--tertiary); stroke-width: 1; stroke-dasharray: 3 3; vector-effect: non-scaling-stroke; }
</style>
