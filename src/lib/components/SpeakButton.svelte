<script lang="ts">
  import { onMount } from 'svelte';
  import { hasJapaneseVoice, speakJapanese } from '$lib/speech';

  /** Japanese text to pronounce */
  export let text: string;
  /** What is being pronounced, for screen readers and the tooltip */
  export let label = 'Listen';

  // Only shown when the system has a Japanese voice
  let available = false;

  onMount(async () => {
    available = await hasJapaneseVoice();
  });
</script>

{#if available && text}
  <button
    type="button"
    class="speak-button"
    title={label}
    aria-label={`${label}: ${text}`}
    on:click|stopPropagation={() => speakJapanese(text)}
  >
    🔊
  </button>
{/if}

<style>
  .speak-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.8rem;
    height: 1.8rem;
    padding: 0;
    margin-left: 0.35rem;
    vertical-align: middle;
    font-size: 0.95rem;
    line-height: 1;
    border-radius: 50%;
    border: 1px solid var(--button-border, #ccc);
    background-color: var(--button-bg, #f3f3f3);
    cursor: pointer;
  }

  .speak-button:hover {
    border-color: var(--accent-color, #396cd8);
  }
</style>
