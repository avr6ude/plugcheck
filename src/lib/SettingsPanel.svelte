<script lang="ts">
  import { createCheckbox, createSelect, melt } from "@melt-ui/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { settings, saveSettings } from "./snapshot.svelte";

  const { elements: { root: launchRoot }, states: { checked: launchChecked } } = createCheckbox({ defaultChecked: settings.launch_at_login, onCheckedChange: ({ next }) => { settings.launch_at_login = next === true; saveSettings(); return next; } });
  const { elements: { root: menuRoot }, states: { checked: menuChecked } } = createCheckbox({ defaultChecked: settings.menu_bar_only, onCheckedChange: ({ next }) => { settings.menu_bar_only = next === true; saveSettings(); return next; } });
  const { elements: { root: notifyRoot }, states: { checked: notifyChecked } } = createCheckbox({ defaultChecked: settings.notifications, onCheckedChange: ({ next }) => { settings.notifications = next === true; saveSettings(); return next; } });
  const { elements: { root: techRoot }, states: { checked: techChecked } } = createCheckbox({ defaultChecked: settings.show_technical, onCheckedChange: ({ next }) => { settings.show_technical = next === true; saveSettings(); return next; } });
  const { elements: { root: wattsRoot }, states: { checked: wattsChecked } } = createCheckbox({ defaultChecked: settings.menu_bar_watts, onCheckedChange: ({ next }) => { settings.menu_bar_watts = next === true; saveSettings(); return next; } });
  const { elements: { root: updRoot }, states: { checked: updChecked } } = createCheckbox({ defaultChecked: settings.update_checks, onCheckedChange: ({ next }) => { settings.update_checks = next === true; saveSettings(); return next; } });

  let cli = $state<"unknown" | "installed" | "missing" | "busy">("unknown");
  let sandboxed = $state(false); // App Store build: no self-updates, no CLI symlink
  onMount(() => { invoke<boolean>("is_sandboxed").then((s) => (sandboxed = s)).catch(() => {}); });
  let cliError = $state("");
  onMount(() => { invoke<boolean>("cli_installed").then((ok) => (cli = ok ? "installed" : "missing")).catch(() => {}); });
  async function installCli() {
    cli = "busy"; cliError = "";
    try { await invoke("install_cli"); cli = "installed"; }
    catch (e) { cli = "missing"; if (!String(e).includes("User canceled")) cliError = String(e); }
  }

  const intervals = [1, 2, 3, 5, 10];
  const label = (s: number) => `Every ${s} second${s === 1 ? "" : "s"}`;
  const { elements: { trigger, menu, option }, states: { selectedLabel, open } } = createSelect<number>({ defaultSelected: { value: settings.poll_secs, label: label(settings.poll_secs) }, onSelectedChange: ({ next }) => { if (next) { settings.poll_secs = next.value; saveSettings(); } return next; } });
</script>

<div>
  <h3 class="list-title">General</h3>
  <div class="group">
    <div class="row"><span id="s-launch"><b>Open at login</b><small>Start PlugCheck when you log in to your Mac.</small></span><button class="switch" aria-labelledby="s-launch" use:melt={$launchRoot}><i class:on={$launchChecked === true}></i></button></div>
    <div class="row"><span id="s-menu"><b>Menu bar only</b><small>Hide the Dock icon and open PlugCheck from the menu bar.</small></span><button class="switch" aria-labelledby="s-menu" use:melt={$menuRoot}><i class:on={$menuChecked === true}></i></button></div>
    <div class="row"><span id="s-watts"><b>Charging watts in the menu bar</b><small>Show live charging power next to the menu-bar icon.</small></span><button class="switch" aria-labelledby="s-watts" use:melt={$wattsRoot}><i class:on={$wattsChecked === true}></i></button></div>
    {#if !sandboxed}<div class="row"><span id="s-upd"><b>Check for updates</b><small>Look for a new release on GitHub every few hours.</small></span><button class="switch" aria-labelledby="s-upd" use:melt={$updRoot}><i class:on={$updChecked === true}></i></button></div>{/if}
  </div>

  <h3 class="list-title">Scanning</h3>
  <div class="group">
    <div class="row">
      <span><b>Refresh</b><small>How often PlugCheck rescans your ports.</small></span>
      <div class="popup">
        <button class="popup-button" use:melt={$trigger}>{$selectedLabel}<svg viewBox="0 0 8 12" width="7" height="11" aria-hidden="true"><path d="M1 4.3 4 1.3l3 3M1 7.7l3 3 3-3" /></svg></button>
        {#if $open}
          <div class="menu" use:melt={$menu}>
            {#each intervals as seconds}
              <button class="item" use:melt={$option({ value: seconds, label: label(seconds) })}>{label(seconds)}</button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
    <div class="row"><span id="s-notify"><b>Plug and unplug notifications</b><small>Tell me when something is connected or removed.</small></span><button class="switch" aria-labelledby="s-notify" use:melt={$notifyRoot}><i class:on={$notifyChecked === true}></i></button></div>
  </div>

  <h3 class="list-title">Advanced</h3>
  <div class="group">
    <div class="row"><span id="s-tech"><b>Show technical details</b><small>Adds a Technical tab with each port’s raw macOS data.</small></span><button class="switch" aria-labelledby="s-tech" use:melt={$techRoot}><i class:on={$techChecked === true}></i></button></div>
    {#if !sandboxed}<div class="row">
      <span><b>Command-line tool</b><small>{cli === "installed" ? "Installed. Run plugcheck --help in Terminal." : "Adds the plugcheck command to Terminal: --text, --json, --watch, --raw."}</small></span>
      {#if cli !== "installed"}<button class="push" onclick={installCli} disabled={cli === "busy"}>{cli === "busy" ? "Installing…" : "Install"}</button>{/if}
    </div>{/if}
  </div>
  {#if cliError}<p class="group-note" role="alert">{cliError}</p>{/if}
</div>

<style>
  .group > .row { min-height: 44px; padding-block: 8px; }
  .row > span { display: grid; gap: 2px; }
  b { font-weight: 400; }
  small { color: var(--muted); font-size: 11px; }

  /* NSSwitch */
  .switch { flex: none; width: 38px; height: 22px; padding: 0; border: 0; border-radius: 11px; background: none; }
  .switch i { position: relative; display: block; width: 100%; height: 100%; border-radius: inherit; background: rgba(0, 0, 0, .1); box-shadow: inset 0 0 0 .5px var(--line); transition: background 160ms ease; }
  @media (prefers-color-scheme: dark) { .switch i { background: rgba(255, 255, 255, .18); } }
  .switch i::after { content: ""; position: absolute; top: 2px; left: 2px; width: 18px; height: 18px; border-radius: 50%; background: #fff; box-shadow: 0 1px 2px rgba(0, 0, 0, .3), 0 0 0 .5px rgba(0, 0, 0, .06); transition: transform 160ms ease; }
  .switch i.on { background: var(--accent); }
  .switch i.on::after { transform: translateX(16px); }

  /* NSPopUpButton */
  .popup { position: relative; flex: none; }
  .popup-button { display: flex; align-items: center; gap: 8px; height: 22px; padding: 0 6px 0 10px; border: 0; border-radius: 5px; background: var(--btn); box-shadow: var(--btn-edge); }
  .popup-button svg { fill: none; stroke: currentColor; stroke-width: 1.4; stroke-linecap: round; stroke-linejoin: round; opacity: .7; }
  .menu { position: absolute; z-index: 10; right: 0; top: 26px; min-width: 100%; padding: 5px; border-radius: 7px; background: var(--group); box-shadow: 0 0 0 .5px var(--line), 0 8px 24px rgba(0, 0, 0, .2); }
  .item { display: block; width: 100%; height: 22px; padding: 0 10px; border: 0; border-radius: 4px; background: none; text-align: left; white-space: nowrap; }
  .item:hover, .item[data-highlighted] { background: var(--accent); color: #fff; }
  @media (prefers-reduced-motion: reduce) { .switch i, .switch i::after { transition: none; } }
</style>
