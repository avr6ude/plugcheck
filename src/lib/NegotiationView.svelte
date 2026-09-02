<script lang="ts">
  import type { Port } from "./snapshot.svelte";
  import { store } from "./snapshot.svelte";

  const occ = $derived((store.snapshot?.ports ?? []).filter((p) => p.occupied));

  const ORDER = ["none", "usb2", "usb3_gen1", "usb3_gen2", "usb4_gen3", "thunderbolt3", "usb4_gen4", "thunderbolt4"];
  function name(s: string): string {
    return (
      {
        none: "—",
        usb2: "USB 2.0",
        usb3_gen1: "USB 3.2 Gen 1 (5 Gb/s)",
        usb3_gen2: "USB 3.2 Gen 2 (10 Gb/s)",
        usb4_gen3: "USB4 Gen 3 (20 Gb/s)",
        usb4_gen4: "USB4 (40 Gb/s)",
        thunderbolt3: "Thunderbolt 3",
        thunderbolt4: "Thunderbolt 4",
        displayport: "DisplayPort",
      }[s] ?? s
    );
  }
  function fastestDevice(p: Port): string {
    let best = "none";
    const walk = (ns: Port["devices"]) => {
      for (const d of ns) {
        if (ORDER.indexOf(d.speed) > ORDER.indexOf(best)) best = d.speed;
        walk(d.children);
      }
    };
    walk(p.devices);
    return best;
  }
  function weakLink(p: Port): "cable" | "port" | "device" | null {
    const dev = fastestDevice(p);
    const active = p.active_transport;
    if (ORDER.indexOf(active) >= ORDER.indexOf(dev)) return null; // running at device max
    // active below device — who's holding it back?
    if (!p.emarker.present && active === "usb2") return "cable";
    if (p.emarker.present && ORDER.indexOf(p.emarker.max_speed) <= ORDER.indexOf(active))
      return "cable";
    return "port";
  }
</script>

<div class="neg">
  {#if occ.length === 0}
    <p class="empty">No cables connected.</p>
  {/if}
  {#each occ as p}
    {@const dev = fastestDevice(p)}
    {@const weak = weakLink(p)}
    <div class="grp">
      <div class="gh">{p.id}</div>
      <table>
        <tbody>
          <tr class:weak={weak === "port"}>
            <td class="k">Mac port</td>
            <td>{p.supported.join(", ") || "—"}</td>
          </tr>
          <tr class:weak={weak === "cable"}>
            <td class="k">Cable</td>
            <td>{p.emarker.present ? name(p.emarker.max_speed) : "Unknown — no e-marker"}</td>
          </tr>
          <tr class:weak={weak === "device"}>
            <td class="k">Device</td>
            <td>{dev !== "none" ? name(dev) : "—"}</td>
          </tr>
          <tr class="row-neg">
            <td class="k">Negotiated</td>
            <td>{p.provisioned.join(", ") || "—"}</td>
          </tr>
        </tbody>
      </table>
      <p class="verdict">
        {#if weak === "cable"}Cable is the weak link.
        {:else if weak === "port"}Mac port is negotiating low.
        {:else}Device maximum — cable and port aren't limiting.{/if}
      </p>
    </div>
  {/each}
</div>

<style>
  .neg {
    display: grid;
    gap: 1rem;
  }
  .empty {
    color: var(--muted);
  }
  .grp {
    border: 0.5px solid var(--line);
    border-radius: 10px;
    background: var(--card);
    padding: 0.7rem 0.85rem;
  }
  .gh {
    font: 0.7rem/1.2 var(--mono);
    color: var(--muted);
    margin-bottom: 0.4rem;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }
  td {
    padding: 0.3rem 0.4rem;
    border-top: 0.5px solid var(--line);
  }
  tr:first-child td {
    border-top: 0;
  }
  .k {
    width: 6.5rem;
    color: var(--muted);
  }
  .row-neg td {
    font-weight: 600;
  }
  tr.weak td {
    background: color-mix(in srgb, var(--warn) 14%, transparent);
  }
  tr.weak .k {
    color: var(--warn-fg);
  }
  .verdict {
    margin: 0.5rem 0 0;
    font-size: 0.82rem;
    color: var(--muted);
  }
</style>
