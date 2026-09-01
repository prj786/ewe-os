<script>
  // Auto-detected, never silently trusted: the suggestion is SHOWN and the
  // user confirms or corrects it (a roaming hotspot once moved a machine to
  // London — the desktop dispatcher and this screen share the same rule).
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { choices } from "../state.js";
  let zones = [], locales = [], confident = false, note = "";
  onMount(async () => {
    zones = await invoke("timezones");
    locales = await invoke("locales");
    if (!$choices.timezone) {
      const s = await invoke("suggest_timezone");
      if (s.zone) {
        $choices.timezone = s.zone;
        $choices.tzConfident = !!s.confident;
        note = s.confident ? "detected from your network — confirm or change it"
                           : "uncertain guess (providers disagree) — please check";
      }
    }
  });
</script>

<h1 class="mb-1 text-2xl font-bold tracking-tight">Time & place</h1>
<p class="mb-6 text-sm text-zinc-400">{note || "Pick your timezone and language."}</p>

<label class="mb-1 block text-sm text-zinc-300" for="tz">Timezone</label>
<select id="tz" class="input mb-4 max-w-96"
        bind:value={$choices.timezone}>
  <option value="" disabled>choose…</option>
  {#each zones as z}<option value={z}>{z}</option>{/each}
</select>

<label class="mb-1 block text-sm text-zinc-300" for="loc">Language / locale</label>
<select id="loc" class="input max-w-96"
        bind:value={$choices.locale}>
  {#each locales as l}<option value={l}>{l}</option>{/each}
</select>
