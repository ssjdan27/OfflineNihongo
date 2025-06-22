<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import './kanji-view.css';

  interface Kanji {
    character: string;
    stroke_count: number;
    grade: number;
    jlpt_level: number;
    frequency: number;
    onyomi: string;
    kunyomi: string;
    meanings: string;
    nanori: string;
  }

  let kanjiList: Kanji[] = [];
  let filteredKanji: Kanji[] = [];
  let loading = true;
  let error = '';
  let activeTab = 'all';
  let activeJlptTab = 'all';

  const tabs = [
    { id: 'all', label: 'All Kanji' },
    { id: 'frequent', label: 'Top 1000 Frequent' },
    { id: 'grade', label: 'By Grade' },
    { id: 'jlpt', label: 'By JLPT Level' }
  ];

  const jlptTabs = [
    { id: 'all', label: 'All JLPT' },
    { id: '1', label: 'JLPT N1' },
    { id: '2', label: 'JLPT N2' },
    { id: '3', label: 'JLPT N3' },
    { id: '4', label: 'JLPT N4' },
    { id: '1-4', label: 'JLPT N1-N4' }
  ];

  onMount(async () => {
    await loadKanji();
  });

  async function loadKanji() {
    try {
      loading = true;
      error = '';
      kanjiList = await invoke('get_all_kanji');
      filterKanji();
    } catch (err) {
      error = `Failed to load kanji: ${err}`;
      console.error('Error loading kanji:', err);
    } finally {
      loading = false;
    }
  }

  function filterKanji() {
    let filtered = [...kanjiList];

    switch (activeTab) {
      case 'all':
        // Show all kanji, no additional filtering
        break;
      
      case 'frequent':
        // Top 1000 most frequent (frequency > 0), sorted by frequency ascending
        filtered = filtered
          .filter(k => k.frequency > 0)
          .sort((a, b) => a.frequency - b.frequency)
          .slice(0, 1000);
        break;
      
      case 'grade':
        // Sort by grade, excluding grade 0
        filtered = filtered
          .filter(k => k.grade > 0)
          .sort((a, b) => a.grade - b.grade);
        break;
      
      case 'jlpt':
        // Filter by JLPT level
        if (activeJlptTab === 'all') {
          filtered = filtered.filter(k => k.jlpt_level > 0);
        } else if (activeJlptTab === '1-4') {
          filtered = filtered.filter(k => k.jlpt_level >= 1 && k.jlpt_level <= 4);
        } else {
          const level = parseInt(activeJlptTab);
          filtered = filtered.filter(k => k.jlpt_level === level);
        }
        // Sort by JLPT level, then by frequency
        filtered.sort((a, b) => {
          if (a.jlpt_level !== b.jlpt_level) {
            return a.jlpt_level - b.jlpt_level;
          }
          return (a.frequency || 9999) - (b.frequency || 9999);
        });
        break;
    }

    filteredKanji = filtered;
  }

  function setActiveTab(tabId: string) {
    activeTab = tabId;
    if (tabId !== 'jlpt') {
      activeJlptTab = 'all';
    }
    filterKanji();
  }

  function setActiveJlptTab(tabId: string) {
    activeJlptTab = tabId;
    filterKanji();
  }

  function goToHome() {
    goto('/');
  }

  function formatMeanings(meanings: string): string {
    if (!meanings) return '';
    return meanings.split(',').slice(0, 3).join(', ');
  }

  function getGradeLabel(grade: number): string {
    if (grade >= 1 && grade <= 6) return `Grade ${grade}`;
    if (grade === 8) return 'Jr. High';
    if (grade === 9) return 'Jinmeiyou';
    if (grade === 10) return 'Jinmeiyou Variant';
    return 'Unknown';
  }

  function getJlptLabel(level: number): string {
    if (level >= 1 && level <= 5) return `N${level}`;
    return '';
  }
</script>

<div class="nav-bar">
  <button class="home-button" on:click={goToHome}>Back to Home</button>
</div>

<main class="kanji-container">
  <h1>Kanji Database</h1>
  
  <!-- Main tabs -->
  <div class="tab-container">
    {#each tabs as tab}
      <button
        class="tab {activeTab === tab.id ? 'active' : ''}"
        on:click={() => setActiveTab(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- JLPT sub-tabs -->
  {#if activeTab === 'jlpt'}
    <div class="sub-tab-container">
      {#each jlptTabs as tab}
        <button
          class="sub-tab {activeJlptTab === tab.id ? 'active' : ''}"
          on:click={() => setActiveJlptTab(tab.id)}
        >
          {tab.label}
        </button>
      {/each}
    </div>
  {/if}

  <!-- Content -->
  {#if loading}
    <div class="loading">Loading kanji...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else}
    <div class="kanji-stats">
      Showing {filteredKanji.length} kanji
      {#if activeTab === 'frequent'}
        (Top 1000 most frequent)
      {:else if activeTab === 'grade'}
        (Sorted by grade)
      {:else if activeTab === 'jlpt'}
        {#if activeJlptTab === 'all'}
          (All JLPT levels)
        {:else if activeJlptTab === '1-4'}
          (JLPT N1-N4)
        {:else}
          (JLPT N{activeJlptTab})
        {/if}
      {/if}
    </div>

    <div class="kanji-grid">
      {#each filteredKanji as kanji}
        <div class="kanji-card">
          <div class="kanji-character">{kanji.character}</div>
          <div class="kanji-info">
            <div class="kanji-readings">
              {#if kanji.onyomi}
                <div class="reading onyomi">
                  <span class="label">On:</span> {kanji.onyomi}
                </div>
              {/if}
              {#if kanji.kunyomi}
                <div class="reading kunyomi">
                  <span class="label">Kun:</span> {kanji.kunyomi}
                </div>
              {/if}
            </div>
            {#if kanji.meanings}
              <div class="meanings">{formatMeanings(kanji.meanings)}</div>
            {/if}
            <div class="kanji-meta">
              {#if kanji.grade > 0}
                <span class="meta-item grade">{getGradeLabel(kanji.grade)}</span>
              {/if}
              {#if kanji.jlpt_level > 0}
                <span class="meta-item jlpt">{getJlptLabel(kanji.jlpt_level)}</span>
              {/if}
              {#if kanji.frequency > 0}
                <span class="meta-item frequency">#{kanji.frequency}</span>
              {/if}
              <span class="meta-item strokes">{kanji.stroke_count} strokes</span>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>
