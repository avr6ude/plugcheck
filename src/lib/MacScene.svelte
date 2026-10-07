<script module lang="ts">
  import type { Port } from "./snapshot.svelte";

  type V = [number, number, number];
  type P2 = [number, number];
  type Side = "left" | "right";
  type Edge = [V, V];
  // `edges` are outlined whenever the face shows. Side faces of an extrusion also know their
  // neighbours: where a neighbour is hidden, the shared vertical edge is part of the silhouette.
  type Poly = { pts: V[]; n: V; cls: string; shade?: boolean; edges?: Edge[]; prev?: Poly; next?: Poly; start?: Edge; end?: Edge };

  const DEG = Math.PI / 180;
  const PITCH = 20 * DEG, CP = Math.cos(PITCH), SP = Math.sin(PITCH);
  const DIST = 1100; // camera distance, mm; lower = stronger perspective
  const YAW: Record<Side, number> = { left: 52, right: -52 };
  const LIGHT: V = norm([-0.35, 0.8, 0.55]);

  // 14" MacBook Pro, mm. x = right, y = up, z = toward the viewer.
  const W = 312.6, D = 221.2, BASE_H = 11, LID_T = 5, LID_H = 215, LID_TILT = 18 * DEG;
  const SIDE_X = W / 2 + 0.15, PORT_Y = 5.5;

  function sub(a: V, b: V): V { return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]; }
  function dot(a: V, b: V) { return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]; }
  function norm(a: V): V { const l = Math.hypot(...a) || 1; return [a[0] / l, a[1] / l, a[2] / l]; }
  function mean(pts: V[]): V { const m: V = [0, 0, 0]; for (const p of pts) { m[0] += p[0]; m[1] += p[1]; m[2] += p[2]; } return [m[0] / pts.length, m[1] / pts.length, m[2] / pts.length]; }
  function newell(pts: V[]): V {
    const n: V = [0, 0, 0];
    pts.forEach((a, i) => { const b = pts[(i + 1) % pts.length]; n[0] += (a[1] - b[1]) * (a[2] + b[2]); n[1] += (a[2] - b[2]) * (a[0] + b[0]); n[2] += (a[0] - b[0]) * (a[1] + b[1]); });
    return norm(n);
  }

  function roundRect(w: number, h: number, r: number, seg = 6): P2[] {
    const pts: P2[] = [], cx = w / 2 - r, cy = h / 2 - r;
    for (const [sx, sy, a0] of [[1, 1, 0], [-1, 1, 90], [-1, -1, 180], [1, -1, 270]])
      for (let i = 0; i <= seg; i++) { const a = (a0 + (90 * i) / seg) * DEG; pts.push([sx * cx + r * Math.cos(a), sy * cy + r * Math.sin(a)]); }
    return pts;
  }

  // Convex extrusion; outward normals found against the solid's centre so winding never matters.
  function prism(outline: P2[], map: (a: number, b: number, t: number) => V, t0: number, t1: number, cls: string): Poly[] {
    const lo = outline.map(([a, b]) => map(a, b, t0)), hi = outline.map(([a, b]) => map(a, b, t1));
    const c = mean([...lo, ...hi]);
    const face = (pts: V[], edges: Edge[]): Poly => { const n = newell(pts); return { pts, n: dot(n, sub(mean(pts), c)) < 0 ? [-n[0], -n[1], -n[2]] as V : n, cls, shade: true, edges }; };
    const ring = (r: V[]): Edge[] => r.map((p, i) => [p, r[(i + 1) % r.length]]);
    const k = lo.length;
    const sides = lo.map((_, i) => { const j = (i + 1) % k; return { ...face([lo[i], lo[j], hi[j], hi[i]], [[lo[i], lo[j]], [hi[i], hi[j]]]), start: [lo[i], hi[i]] as Edge, end: [lo[j], hi[j]] as Edge }; });
    sides.forEach((f, i) => { f.prev = sides[(i + k - 1) % k]; f.next = sides[(i + 1) % k]; });
    return [face(lo, ring(lo)), face(hi, ring(hi)), ...sides];
  }
  function decal(outline: P2[], map: (a: number, b: number) => V, n: V, cls: string): Poly {
    return { pts: outline.map(([a, b]) => map(a, b)), n, cls };
  }

  const lidPt = (u: number, v: number, w: number): V =>
    [u, BASE_H + 1.5 + v * Math.cos(LID_TILT) + w * Math.sin(LID_TILT), -D / 2 + 4.6 - v * Math.sin(LID_TILT) + w * Math.cos(LID_TILT)];
  const LID_N: V = [0, Math.sin(LID_TILT), Math.cos(LID_TILT)];
  const UP: V = [0, 1, 0];
  const onTop = (y = BASE_H + 0.05) => (a: number, b: number): V => [a, y, b];

  // Keyboard: row height (in key units) and key widths. Last row ends in the inverted-T arrow cluster.
  const KEY = 18.6, GAP = 0.12;
  const ROWS: [number, number[]][] = [
    [0.55, Array(14).fill(14.5 / 14)],
    [1, [...Array(13).fill(1), 1.5]],
    [1, [1.5, ...Array(13).fill(1)]],
    [1, [1.8, ...Array(11).fill(1), 1.7]],
    [1, [2.3, ...Array(10).fill(1), 2.2]],
    [1, [1, 1, 1, 1.25, 5, 1.25, 1]],
  ];
  function keyboard(): Poly[] {
    const keys: Poly[] = [], x0 = (-14.5 * KEY) / 2;
    const key = (x: number, z: number, w: number, d: number) =>
      keys.push(decal([[x + GAP * KEY / 2, z + GAP * KEY / 2], [x + w - GAP * KEY / 2, z + GAP * KEY / 2], [x + w - GAP * KEY / 2, z + d - GAP * KEY / 2], [x + GAP * KEY / 2, z + d - GAP * KEY / 2]], onTop(), UP, "key"));
    let z = -D / 2 + 11;
    for (const [h, widths] of ROWS) {
      let x = x0;
      for (const w of widths) { key(x, z, w * KEY, h * KEY); x += w * KEY; }
      if (widths.length === 7) { // arrows: ← (↑ over ↓) →, half height
        const half = KEY / 2;
        key(x, z + half, KEY, half); key(x + KEY, z, KEY, half); key(x + KEY, z + half, KEY, half); key(x + 2 * KEY, z + half, KEY, half);
      }
      z += h * KEY;
    }
    return keys;
  }

  // Physical openings. Only `id`'d ones are tracked ports; the rest are drawn for realism.
  type Opening = { side: Side; z: number; w: number; h: number; label?: string; fallbackId?: string; match?: (p: Port) => boolean };
  const isUsbC = (n: number) => (p: Port) => p.kind !== "MagSafe 3" && p.id.endsWith(`@${n}`);
  const OPENINGS: Opening[] = [
    { side: "left", z: -72, w: 13, h: 3.4, label: "MagSafe 3", fallbackId: "port-magsafe", match: (p) => p.kind === "MagSafe 3" },
    { side: "left", z: -48, w: 8.4, h: 2.6, label: "USB-C 1", fallbackId: "port-usbc1", match: isUsbC(1) },
    { side: "left", z: -32, w: 8.4, h: 2.6, label: "USB-C 2", fallbackId: "port-usbc2", match: isUsbC(2) },
    { side: "left", z: 62, w: 3.6, h: 3.6 },
    { side: "right", z: -62, w: 14, h: 4.5 },
    { side: "right", z: -38, w: 8.4, h: 2.6, label: "USB-C 3", fallbackId: "port-usbc3", match: isUsbC(3) },
    { side: "right", z: 40, w: 24, h: 2 },
  ];
  const sideX = (s: Side) => (s === "left" ? -SIDE_X : SIDE_X);
  const sideN = (s: Side): V => [s === "left" ? -1 : 1, 0, 0];

  const BODY = [
    ...prism(roundRect(W, LID_H, 10), (a, b, t) => lidPt(a, b + LID_H / 2, t), -LID_T, 0, "shell"),
    decal(roundRect(W - 12.6, 195, 6), (a, b) => lidPt(a, b + 111.5, 0.2), LID_N, "screen"),
    decal(roundRect(32, 6, 1.5), (a, b) => lidPt(a, b + 206, 0.3), LID_N, "bezel"),
    ...prism(roundRect(W, D, 12), (a, b, t) => [a, t, b], 0, BASE_H, "shell"),
    ...keyboard(),
    decal(roundRect(140, 86, 6), (a, b) => [a, BASE_H + 0.05, b + D / 2 - 10 - 43], UP, "trackpad"), // palm rest, 10 mm from the front edge
    ...OPENINGS.map((o) => decal(roundRect(o.w, o.h, o.h / 2), (a, b) => [sideX(o.side), PORT_Y + b, o.z + a], sideN(o.side), o.label ? "port" : "opening")),
  ];

  function rot([x, y, z]: V, yaw: number): V {
    const c = Math.cos(yaw * DEG), s = Math.sin(yaw * DEG);
    const x1 = x * c + z * s, z1 = -x * s + z * c;
    return [x1, y * CP - z1 * SP, y * SP + z1 * CP];
  }
  const cam = (p: V, yaw: number) => rot([p[0], p[1] - 60, p[2]], yaw);
  function persp([x, y, z]: V): P2 { const k = DIST / (DIST - z); return [x * k, -y * k]; }

  // One bounding box for every pose between the two sides, so the model never resizes mid-turn.
  const BOUNDS = (() => {
    let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
    for (let yaw = YAW.right; yaw <= YAW.left; yaw += 4)
      for (const f of BODY) if (f.cls === "shell") for (const p of f.pts) {
        const [x, y] = persp(cam(p, yaw));
        x0 = Math.min(x0, x); x1 = Math.max(x1, x); y0 = Math.min(y0, y); y1 = Math.max(y1, y);
      }
    return { cx: (x0 + x1) / 2, cy: (y0 + y1) / 2, w: x1 - x0, h: y1 - y0 };
  })();

  export function portLabel(p: Port): string {
    return OPENINGS.find((o) => o.match?.(p))?.label ?? p.kind;
  }

  export type SidePort = { label: string; fallbackId: string; port: Port | undefined; z: number; side: Side };
  export function sidePorts(ports: Port[], side: Side): SidePort[] {
    return OPENINGS.filter((o) => o.side === side && o.label).map((o) => ({ label: o.label!, fallbackId: o.fallbackId!, port: ports.find(o.match!), z: o.z, side }));
  }
</script>

<script lang="ts">
  import { createToggleGroup, melt } from "@melt-ui/svelte";
  import { untrack } from "svelte";
  import { Tween } from "svelte/motion";
  import { cubicInOut } from "svelte/easing";
  import { writable } from "svelte/store";

  let { ports, selectedPortId, side, onSelectPort, onSideChange }: {
    ports: Port[];
    selectedPortId: string | null;
    side: Side;
    onSelectPort: (id: string) => void;
    onSideChange: (side: Side) => void;
  } = $props();

  const reduce = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  const yaw = new Tween(untrack(() => YAW[side]), { duration: reduce ? 0 : 650, easing: cubicInOut });
  $effect(() => { const t = YAW[side]; untrack(() => (yaw.target = t)); });
  const settled = $derived(Math.abs(yaw.current - YAW[side]) < 0.5);

  const sideValue = writable<string>(untrack(() => side));
  $effect(() => sideValue.set(side));
  const { elements: { root, item } } = createToggleGroup({
    type: "single",
    value: sideValue,
    onValueChange: ({ curr, next }) => {
      if (next !== "left" && next !== "right") return curr; // a single-choice switch can't be emptied
      onSideChange(next);
      return next;
    },
  });

  let vw = $state(0), vh = $state(0);
  const LABEL_W = $derived(vw < 420 ? 104 : 120);
  const LABEL_H = 40, LANE = 36; // LANE: room for the leader line between label and model
  const scale = $derived(Math.max(0, Math.min((vw - LABEL_W - LANE) / BOUNDS.w, (vh - 24) / BOUNDS.h)));
  // Slide the model away from the labelled side, in step with the turn.
  const shift = $derived((yaw.current / YAW.left) * (LABEL_W + LANE) / 2);
  function toPx(p: V): P2 {
    const [x, y] = persp(cam(p, yaw.current));
    return [vw / 2 + shift + (x - BOUNDS.cx) * scale, vh / 2 + (y - BOUNDS.cy) * scale];
  }

  const pt = (p: V) => toPx(p).map((n) => n.toFixed(1)).join(",");
  const scene = $derived.by(() => {
    const shown = new Map<Poly, V>();
    for (const f of BODY) {
      const nc = rot(f.n, yaw.current);
      if (dot(nc, sub([0, 0, DIST], cam(mean(f.pts), yaw.current))) > 0) shown.set(f, nc);
    }
    const faces: { cls: string; d: string; fill?: string }[] = [];
    let edges = "";
    const line = ([a, b]: Edge) => (edges += `M${pt(a)}L${pt(b)}`);
    for (const [f, nc] of shown) {
      faces.push({ cls: f.cls, d: f.pts.map(pt).join(" "), fill: f.shade ? `color-mix(in srgb, var(--shell-hi) ${Math.round(100 * Math.max(0, dot(nc, LIGHT)))}%, var(--shell-lo))` : undefined });
      f.edges?.forEach(line);
      if (f.prev && !shown.has(f.prev)) line(f.start!);
      if (f.next && !shown.has(f.next)) line(f.end!);
    }
    return { faces, edges };
  });

  const entries = $derived(sidePorts(ports, side));
  const labels = $derived.by(() => {
    const anchors = entries.map((e) => ({ e, at: toPx([sideX(side), PORT_Y, e.z]) })).sort((a, b) => a.at[1] - b.at[1]);
    const total = anchors.length * LABEL_H + (anchors.length - 1) * 8;
    const meanY = anchors.reduce((s, a) => s + a.at[1], 0) / (anchors.length || 1);
    const top = Math.min(Math.max(meanY - total / 2, 0), vh - total);
    return anchors.map(({ e, at }, i) => {
      const y = top + i * (LABEL_H + 8) + LABEL_H / 2;
      const edge = side === "left" ? LABEL_W : vw - LABEL_W, elbow = edge + (side === "left" ? 12 : -12);
      return { e, id: e.port?.id ?? e.fallbackId, at, y, x: side === "left" ? 0 : vw - LABEL_W, line: `${edge},${y} ${elbow},${y} ${at[0]},${at[1]}` };
    });
  });
</script>

<section class="scene" aria-label="MacBook port selector">
  <div class="scene-top">
    <span id="side-label">Side</span>
    <div class="side-toggle" use:melt={$root} aria-labelledby="side-label">
      <button use:melt={$item("left")}>Left</button>
      <button use:melt={$item("right")}>Right</button>
    </div>
  </div>

  <div class="viewport" bind:clientWidth={vw} bind:clientHeight={vh}>
    {#if vw && vh}
      <svg width={vw} height={vh} viewBox="0 0 {vw} {vh}" aria-hidden="true">
        <defs>
          <linearGradient id="screen-glass" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stop-color="#2b3036" /><stop offset=".55" stop-color="#121417" /><stop offset="1" stop-color="#0b0c0e" />
          </linearGradient>
        </defs>
        {#each scene.faces as f}<polygon class={f.cls} points={f.d} style:fill={f.fill} style:stroke={f.fill} />{/each}
        <path class="edges" d={scene.edges} />
        {#if settled}
          {#each labels as l (l.id)}
            {@const on = selectedPortId === l.id}
            <g class="callout" class:on class:live={l.e.port?.occupied}>
              <polyline points={l.line} />
              <circle cx={l.at[0]} cy={l.at[1]} r="4.5" />
            </g>
          {/each}
        {/if}
      </svg>
      {#if settled}
        {#each labels as l (l.id)}
          <button
            class="port-label"
            class:on={selectedPortId === l.id}
            class:live={l.e.port?.occupied}
            style:left="{l.x}px"
            style:top="{l.y}px"
            style:width="{LABEL_W}px"
            aria-pressed={selectedPortId === l.id}
            onclick={() => onSelectPort(l.id)}
          >
            <b>{l.e.label}</b>
            <small><i aria-hidden="true"></i>{l.e.port?.occupied ? "In use" : "Not connected"}</small>
          </button>
        {/each}
      {/if}
    {/if}
  </div>
</section>

<style>
  /* Silver aluminium in light appearance, space grey in dark. */
  .scene { --shell-hi: #f6f7f8; --shell-lo: #b4b8bd; --edge: rgba(0, 0, 0, .42); --key: #26272a; --key-line: rgba(255, 255, 255, .08); --pad: #e6e8ea; --hole: #1c1d1f;
    display: grid; grid-template-rows: auto minmax(0, 1fr); height: 100%; min-height: 240px; }
  @media (prefers-color-scheme: dark) {
    .scene { --shell-hi: #6b6f74; --shell-lo: #2b2d30; --edge: rgba(255, 255, 255, .42); --key: #0e0f10; --key-line: rgba(255, 255, 255, .12); --pad: #3a3c40; --hole: #050506; }
  }
  .scene-top { display: flex; align-items: center; justify-content: flex-end; gap: 8px; padding: 4px 0 8px; color: var(--muted); }
  /* NSSegmentedControl, same as the inspector's */
  .side-toggle { display: inline-flex; gap: 2px; padding: 2px; border-radius: 7px; background: var(--fill); }
  .side-toggle button { width: 64px; height: 22px; border: 0; border-radius: 5px; background: none; font-size: 13px; }
  .side-toggle :global(button[data-state="on"]) { background: var(--btn); box-shadow: var(--btn-edge); }
  .viewport { position: relative; min-height: 0; overflow: hidden; }
  svg { position: absolute; inset: 0; display: block; }
  polygon { stroke-linejoin: round; }
  .shell { stroke-width: .8; } /* stroked in its own fill, to hide seams between facets */
  .edges { fill: none; stroke: var(--edge); stroke-width: .8; stroke-linejoin: round; stroke-linecap: round; }
  .screen { fill: url(#screen-glass); }
  .bezel { fill: #08090a; }
  .key { fill: var(--key); stroke: var(--key-line); stroke-width: .5; }
  .trackpad { fill: var(--pad); stroke: var(--edge); stroke-width: .5; }
  .opening { fill: var(--hole); }
  .port { fill: var(--hole); stroke: var(--accent); stroke-width: 1; }
  .callout polyline { fill: none; stroke: var(--tertiary); stroke-width: 1; }
  .callout circle { fill: var(--group); stroke: var(--muted); stroke-width: 1.5; }
  .callout.live circle { fill: var(--ok); stroke: var(--group); }
  .callout.on polyline { stroke: var(--accent); }
  .callout.on circle { stroke: var(--accent); stroke-width: 2; }
  /* Port chips: popover-like, accent-filled when selected, like a selected list row */
  .port-label { position: absolute; display: grid; gap: 1px; height: 40px; padding: 0 10px; transform: translateY(-50%); border: 0; border-radius: 8px; background: var(--group); box-shadow: 0 0 0 .5px var(--line); text-align: left; align-content: center; }
  .port-label.on { background: var(--accent); color: #fff; }
  .port-label b { font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .port-label small { display: flex; align-items: center; gap: 4px; color: var(--muted); font-size: 11px; white-space: nowrap; }
  .port-label small i { width: 6px; height: 6px; border-radius: 50%; background: var(--tertiary); }
  .port-label.live small i { background: var(--ok); }
  .port-label.on small { color: rgba(255, 255, 255, .8); }
  .port-label.on small i { box-shadow: 0 0 0 1px #fff; }
  /* Inactive window: selection goes unemphasised grey, as in AppKit lists. */
  :global(html.inactive) .port-label.on { background: var(--selected); color: var(--fg); }
  :global(html.inactive) .port-label.on small { color: var(--muted); }
  :global(html.inactive) .callout.on polyline, :global(html.inactive) .callout.on circle { stroke: var(--muted); }
</style>
