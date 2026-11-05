<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { getVersion } from '@tauri-apps/api/app';
  import './settings.css';

  let darkMode = false;
  let initialized = false;
  let appVersion = '';

  onMount(async () => {
    if (browser) {
      darkMode = localStorage.getItem('theme') === 'dark';
      applyTheme(darkMode);
      initialized = true;
    }
    try {
      appVersion = await getVersion();
    } catch (err) {
      console.error('Failed to read app version:', err);
    }
  });

  function applyTheme(isDark: boolean) {
    if (browser) {
      document.documentElement.classList.toggle('dark', isDark);
    }
  }

  function handleDarkModeChange() {
    if (browser && initialized) {
      localStorage.setItem('theme', darkMode ? 'dark' : 'light');
      applyTheme(darkMode);
    }
  }

  // Watch for changes to darkMode variable
  $: darkMode, initialized && handleDarkModeChange();

  function goToHome() {
    goto('/');
  }
</script>

<div class="nav-bar">
  <button class="home-button" on:click={goToHome}>Back to Home</button>
</div>

<main class="settings-container">
  <h1>Settings</h1>

  <div class="settings-section">
    <h2>Appearance</h2>

    <div class="setting-item">
      <div class="setting-info">
        <h3>Dark Mode</h3>
        <p>Switch between light and dark themes</p>
      </div>
      <label class="toggle-switch">
        <input type="checkbox" bind:checked={darkMode} aria-label="Dark mode" />
        <span class="slider"></span>
      </label>
    </div>
  </div>

  <div class="settings-section">
    <h2>About</h2>
    <div class="setting-item">
      <div class="setting-info">
        <h3>Version</h3>
        <p>{appVersion || '—'}</p>
      </div>
    </div>
    <div class="setting-item">
      <div class="setting-info">
        <h3>Dictionary Data</h3>
        <p>
          KANJIDIC2 by Jim Breen and the
          <a href="https://www.edrdg.org/wiki/index.php/KANJIDIC_Project" target="_blank" rel="noopener noreferrer">EDRDG</a>
          (CC BY-SA 4.0)
        </p>
      </div>
    </div>
    <div class="setting-item">
      <div class="setting-info">
        <h3>Stroke Order Data</h3>
        <p>
          <a href="https://kanjivg.tagaini.net/" target="_blank" rel="noopener noreferrer">KanjiVG</a>
          by Ulrich Apel (CC BY-SA 3.0) and
          <a href="https://github.com/parsimonhi/animCJK" target="_blank" rel="noopener noreferrer">AnimCJK</a>
          by François Mizessyn (LGPL 3.0)
        </p>
      </div>
    </div>
    <div class="setting-item">
      <div class="setting-info">
        <h3>Developer</h3>
        <p>Daniel Garza ダニエル</p>
      </div>
    </div>
    <div class="setting-item">
      <div class="setting-info">
        <h3>Source Code</h3>
        <p>
          <a href="https://github.com/ssjdan27/OfflineNihongo" target="_blank" rel="noopener noreferrer">
            View on GitHub
          </a>
        </p>
      </div>
    </div>
  </div>
</main>
