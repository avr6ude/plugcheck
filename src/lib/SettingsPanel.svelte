<script lang="ts">
  import { settings, saveSettings } from "./snapshot.svelte";
  let { onClose }: { onClose: () => void } = $props();

  async function commit() {
    await saveSettings();
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />
<!-- svelte-ignore a11y_click_events_have_key_events -- Escape + Done button are the keyboard paths -->
<div class="backdrop" onclick={onClose} role="presentation">
  <div class="panel" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Settings" tabindex="-1">
    <header>
      <strong>Settings</strong>
      <button onclick={onClose}>Done</button>
    </header>

    <label class="row">
      <input type="checkbox" bind:checked={settings.notifications} onchange={commit} />
      <span>
        <b>Notifications</b>
        <small>Alert when a device connects, disconnects, or a link is limited.</small>
      </span>
    </label>

    <label class="row">
      <input type="checkbox" bind:checked={settings.launch_at_login} onchange={commit} />
      <span><b>Launch at login</b></span>
    </label>

    <label class="row">
      <input type="checkbox" bind:checked={settings.menu_bar_only} onchange={commit} />
      <span>
        <b>Menu-bar only</b>
        <small>Hide the Dock icon; reach the app from the menu-bar tray.</small>
      </span>
    </label>

    <label class="row">
      <span><b>Refresh interval</b></span>
      <select bind:value={settings.poll_secs} onchange={commit}>
        <option value={1}>1 second</option>
        <option value={2}>2 seconds</option>
        <option value={3}>3 seconds</option>
        <option value={5}>5 seconds</option>
        <option value={10}>10 seconds</option>
      </select>
    </label>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
  }
  .panel {
    background: var(--card);
    border: 1px solid var(--line);
    border-radius: 12px;
    width: 100%;
    max-width: 420px;
    padding: 1rem 1.1rem 1.2rem;
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.9rem;
  }
  header button {
    font-size: 0.78rem;
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 7px;
    padding: 0.25rem 0.7rem;
    color: var(--fg);
    cursor: pointer;
  }
  .row {
    display: flex;
    gap: 0.6rem;
    align-items: flex-start;
    padding: 0.5rem 0;
    border-top: 1px solid var(--line);
  }
  .row span {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  .row small {
    color: var(--muted);
    font-size: 0.75rem;
  }
  .row select {
    margin-left: auto;
  }
  input[type="checkbox"] {
    margin-top: 0.15rem;
  }
</style>
