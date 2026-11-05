<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import '../app.css';

  // Initialize theme on app load
  onMount(() => {
    if (browser) {
      const savedTheme = localStorage.getItem('theme');
      if (savedTheme === 'dark') {
        document.documentElement.classList.add('dark');
      }
    }
  });

  // The app's webview doesn't open new windows, so send external links to
  // the system browser instead.
  function handleClick(event: MouseEvent) {
    const target = event.target as Element | null;
    const link = target?.closest?.('a[href]') as HTMLAnchorElement | null;
    if (!link || !/^https?:\/\//.test(link.href) || link.origin === window.location.origin) {
      return;
    }
    event.preventDefault();
    openUrl(link.href).catch((err) => console.error('Failed to open link:', err));
  }
</script>

<svelte:document on:click={handleClick} />

<slot />
