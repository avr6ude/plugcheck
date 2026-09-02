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

<div class="pm">
  <div class="big">
    <span class="n">{cur.toFixed(0)}</span><span class="u">W</span>
    {#if ceiling}<span class="of">now · {ceiling} W adapter</span>{/if}
  </div>

  <svg viewBox="0 0 {W} {H}" preserveAspectRatio="none" class="chart">
    {#if ceilY >= 0}
      <line x1="0" y1={ceilY} x2={W} y2={ceilY} class="ceil" />
    {/if}
    <path d={area()} class="fill" />
    <path d={line()} class="stroke" />
  </svg>
  <div class="axis"><span>peak {peak.toFixed(0)} W</span><span>{pts.length} samples</span></div>

  <dl>
    {#if charger}
      <div>
        <dt>Adapter</dt>
        <dd>{charger.watts ? `${charger.watts} W` : "—"}</dd>
      </div>
      <div>
        <dt>Negotiated</dt>
        <dd>
          {charger.negotiated_volts != null && charger.negotiated_amps != null
            ? `${charger.negotiated_volts.toFixed(0)} V / ${charger.negotiated_amps.toFixed(2)} A`
            : "—"}
        </dd>
      </div>
      <div>
        <dt>Into battery now</dt>
        <dd>{charger.live_watts != null ? `${charger.live_watts.toFixed(0)} W` : "—"}</dd>
      </div>
      <div>
        <dt>Battery</dt>
        <dd>
          {charger.battery_percent != null ? `${charger.battery_percent}%` : "—"}
          {#if charger.minutes_to_full}· full in {Math.floor(charger.minutes_to_full / 60)}h {charger.minutes_to_full % 60}m{/if}
        </dd>
      </div>
      <div>
        <dt>State</dt>
        <dd>{charger.fully_charged ? "Full" : charger.is_charging ? "Charging" : "Not charging"}</dd>
      </div>
    {:else}
      <div><dt>Adapter</dt><dd>No charger connected</dd></div>
    {/if}
  </dl>
</div>

<style>
  .pm {
    display: grid;
    gap: 0.8rem;
  }
  .big {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }
  .n {
    font-size: 2.4rem;
    font-weight: 300;
    font-variant-numeric: tabular-nums;
  }
  .u {
    font-size: 1rem;
    color: var(--muted);
  }
  .of {
    color: var(--muted);
    font-size: 0.8rem;
    margin-left: 0.4rem;
  }
  .chart {
    width: 100%;
    height: 130px;
    border: 0.5px solid var(--line);
    border-radius: 8px;
    background: var(--card);
  }
  .stroke {
    fill: none;
    stroke: var(--accent);
    stroke-width: 1.6;
    vector-effect: non-scaling-stroke;
  }
  .fill {
    fill: color-mix(in srgb, var(--accent) 14%, transparent);
    stroke: none;
  }
  .ceil {
    stroke: var(--warn);
    stroke-width: 1;
    stroke-dasharray: 3 3;
    vector-effect: non-scaling-stroke;
  }
  .axis {
    display: flex;
    justify-content: space-between;
    font-size: 0.7rem;
    color: var(--muted);
    margin-top: -0.5rem;
  }
  dl {
    margin: 0;
    display: grid;
    gap: 0.15rem;
  }
  dl > div {
    display: flex;
    gap: 0.7rem;
    font-size: 0.86rem;
  }
  dt {
    flex: none;
    width: 10rem;
    color: var(--muted);
  }
  dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }
</style>
