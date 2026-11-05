<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  import { getVersion } from '@tauri-apps/api/app';
  import { invoke } from '@tauri-apps/api/core';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import {
    SESSION_LIMIT_OPTIONS,
    getSessionLimits,
    setSessionLimits,
    getRecallReviews,
    setRecallReviews,
    type SessionLimits
  } from '$lib/preferences';
  import './settings.css';

  interface ImportSummary {
    cards: number;
    review_days: number;
    best_times: number;
  }

  let darkMode = false;
  let initialized = false;
  let appVersion = '';
  let limits: SessionLimits = getSessionLimits();
  let recallReviews = getRecallReviews();

  // Data management
  let busy = false;
  let dataMessage = '';
  let dataError = '';
  let lastSavedPath = ''; // shown with a "Show in folder" button
  let pendingImport: { name: string; contents: string } | null = null;
  let confirmingReset = false;
  let fileInput: HTMLInputElement | null = null;

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

  function updateLimits() {
    setSessionLimits(limits);
  }

  function clearDataStatus() {
    dataMessage = '';
    dataError = '';
    lastSavedPath = '';
  }

  async function exportBackup() {
    clearDataStatus();
    pendingImport = null;
    confirmingReset = false;
    busy = true;
    try {
      lastSavedPath = await invoke('export_backup');
      dataMessage = `Backup saved to ${lastSavedPath}`;
    } catch (err) {
      dataError = `Export failed: ${err}`;
    } finally {
      busy = false;
    }
  }

  function chooseImportFile() {
    clearDataStatus();
    confirmingReset = false;
    fileInput?.click();
  }

  async function handleFileChosen(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = ''; // allow choosing the same file again
    if (!file) return;
    try {
      pendingImport = { name: file.name, contents: await file.text() };
    } catch (err) {
      dataError = `Could not read ${file.name}: ${err}`;
    }
  }

  async function confirmImport() {
    if (!pendingImport) return;
    const { name, contents } = pendingImport;
    pendingImport = null;
    clearDataStatus();
    busy = true;
    try {
      const summary: ImportSummary = await invoke('import_backup', { contents });
      dataMessage =
        `Restored ${summary.cards} flashcards, ${summary.review_days} days of review history ` +
        `and ${summary.best_times} best times from ${name}. Your previous data was saved in the app's backups folder.`;
    } catch (err) {
      dataError = `Import failed: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function resetFlashcards() {
    clearDataStatus();
    pendingImport = null;
    if (!confirmingReset) {
      confirmingReset = true;
      return;
    }
    confirmingReset = false;
    busy = true;
    try {
      lastSavedPath = await invoke('reset_flashcards');
      dataMessage = `Flashcards and review history were reset. A copy of the old data is in ${lastSavedPath}`;
    } catch (err) {
      dataError = `Reset failed: ${err}`;
    } finally {
      busy = false;
    }
  }

  async function showInFolder() {
    try {
      await revealItemInDir(lastSavedPath);
    } catch (err) {
      dataError = `Could not open the folder: ${err}`;
    }
  }

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
    <h2>Flashcards</h2>

    <div class="setting-item">
      <div class="setting-info">
        <h3>Reviews per session</h3>
        <p>Most due cards to review in one study session</p>
      </div>
      <select class="setting-select" bind:value={limits.dueLimit} on:change={updateLimits} aria-label="Reviews per session">
        {#each SESSION_LIMIT_OPTIONS as option}
          <option value={option}>{option}</option>
        {/each}
      </select>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <h3>New cards per session</h3>
        <p>Most new cards to learn in one study session</p>
      </div>
      <select class="setting-select" bind:value={limits.newLimit} on:change={updateLimits} aria-label="New cards per session">
        {#each SESSION_LIMIT_OPTIONS as option}
          <option value={option}>{option}</option>
        {/each}
      </select>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <h3>Kanji recall reviews</h3>
        <p>Also ask "which kanji means…?" so you practice recalling the kanji itself</p>
      </div>
      <label class="toggle-switch">
        <input
          type="checkbox"
          bind:checked={recallReviews}
          on:change={() => setRecallReviews(recallReviews)}
          aria-label="Kanji recall reviews"
        />
        <span class="slider"></span>
      </label>
    </div>
  </div>

  <div class="settings-section">
    <h2>Your Data</h2>

    <div class="setting-item">
      <div class="setting-info">
        <h3>Export backup</h3>
        <p>Save your flashcards, review history and kana game best times to a file in Downloads</p>
      </div>
      <button class="setting-button" on:click={exportBackup} disabled={busy}>Export</button>
    </div>

    <div class="setting-item">
      <div class="setting-info">
        <h3>Import backup</h3>
        <p>Replace your current progress with a backup file</p>
      </div>
      <button class="setting-button" on:click={chooseImportFile} disabled={busy}>Import…</button>
      <input
        bind:this={fileInput}
        type="file"
        accept=".json,application/json"
        class="hidden-file-input"
        on:change={handleFileChosen}
      />
    </div>

    {#if pendingImport}
      <div class="confirm-box">
        <p>
          Replace your current flashcards, review history and best times with
          <strong>{pendingImport.name}</strong>? A copy of your current data is saved first.
        </p>
        <div class="confirm-actions">
          <button class="setting-button danger" on:click={confirmImport}>Replace my data</button>
          <button class="setting-button" on:click={() => (pendingImport = null)}>Cancel</button>
        </div>
      </div>
    {/if}

    <div class="setting-item">
      <div class="setting-info">
        <h3>Reset flashcards</h3>
        <p>Delete all flashcards and review history (kana game best times are kept)</p>
      </div>
      <button class="setting-button {confirmingReset ? 'danger' : ''}" on:click={resetFlashcards} disabled={busy}>
        {confirmingReset ? 'Click again to reset' : 'Reset…'}
      </button>
    </div>

    {#if dataMessage}
      <div class="data-message" role="status">
        {dataMessage}
        {#if lastSavedPath}
          <button class="link-button" on:click={showInFolder}>Show in folder</button>
        {/if}
      </div>
    {/if}
    {#if dataError}
      <div class="data-error" role="alert">{dataError}</div>
    {/if}
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
