<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import './settings.css';

  let darkMode = false;
  let initialized = false;

  // Load dark mode preference from localStorage
  onMount(() => {
    if (browser) {
      const savedTheme = localStorage.getItem('theme');
      darkMode = savedTheme === 'dark';
      console.log('Loading theme:', savedTheme, 'darkMode:', darkMode);
      applyTheme(darkMode);
      initialized = true;
    }
  });

  // Apply theme to the document
  function applyTheme(isDark: boolean) {
    if (browser) {
      console.log('Applying theme:', isDark);
      if (isDark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
      console.log('Document classes:', document.documentElement.className);
    }
  }

  // Handle dark mode changes
  function handleDarkModeChange() {
    if (browser && initialized) {
      console.log('Handling dark mode change:', darkMode);
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
        <input 
          type="checkbox" 
          bind:checked={darkMode}
        />
        <span class="slider"></span>
      </label>
    </div>
  </div>

  <div class="settings-section">
    <h2>About</h2>
    <div class="setting-item">
      <div class="setting-info">
        <h3>Version</h3>
        <p>0.1.0</p>
      </div>
    </div>
    <div class="setting-item">
      <div class="setting-info">
        <h3>Dictionary Data</h3>
        <p>KANJIDIC2 by Jim Breen and the EDRDG</p>
      </div>
    </div>
  </div>
</main>
