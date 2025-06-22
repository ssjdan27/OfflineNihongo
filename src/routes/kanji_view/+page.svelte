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
  let filteringLoading = false;
  let error = '';
  let activeTab = 'all';
  let activeJlptTab = 'all';
  let activeStrokeTab = 'all';
  let searchQuery = '';

  const tabs = [
    { id: 'all', label: 'All Kanji' },
    { id: 'frequent', label: 'Top 1000 Frequent' },
    { id: 'grade', label: 'By Grade' },
    { id: 'jlpt', label: 'By JLPT Level' },
    { id: 'strokes', label: 'By Stroke Count' }
  ];

  const jlptTabs = [
    { id: 'all', label: 'All JLPT' },
    { id: '1', label: 'JLPT N1' },
    { id: '2', label: 'JLPT N2' },
    { id: '3', label: 'JLPT N3' },
    { id: '4', label: 'JLPT N4' }
  ];

  const strokeTabs = [
    { id: 'all', label: 'All Strokes' },
    { id: '1-5', label: '1-5 Strokes' },
    { id: '6-10', label: '6-10 Strokes' },
    { id: '11-20', label: '11-20 Strokes' },
    { id: '20+', label: '20+ Strokes' }
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
    // Show loading animation for larger datasets
    filteringLoading = true;
    
    // Use setTimeout to allow UI to update and show loading animation
    setTimeout(() => {
      const startTime = Date.now();
      
      let filtered = [...kanjiList];

      // Apply search filter first if there's a search query
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase().trim();
        filtered = filtered.filter(kanji => {
          // Search in meanings
          if (kanji.meanings && kanji.meanings.toLowerCase().includes(query)) {
            return true;
          }
          // Also search in character itself
          if (kanji.character.includes(query)) {
            return true;
          }
          // Search in readings (onyomi and kunyomi)
          if (kanji.onyomi && kanji.onyomi.toLowerCase().includes(query)) {
            return true;
          }
          if (kanji.kunyomi && kanji.kunyomi.toLowerCase().includes(query)) {
            return true;
          }
          return false;
        });
      }

      switch (activeTab) {
        case 'all':
          // Show all kanji, no additional filtering
          break;
        
        case 'frequent':
          // Top 1000 most frequent (frequency > 0), sorted by frequency ascending - we leave out those with frequency 0 for now since they don't seem to have a categorization
          filtered = filtered
            .filter(k => k.frequency > 0)
            .sort((a, b) => a.frequency - b.frequency)
            .slice(0, 1000);
          break;
        
        case 'grade':
          // Sort by grade, excluding grade 0 since those are not categorized
          filtered = filtered
            .filter(k => k.grade > 0)
            .sort((a, b) => a.grade - b.grade);
          break;
        
        case 'jlpt':
          // Filter by JLPT level - leave out those with jlpt_level 0 since they are not categorized
          if (activeJlptTab === 'all') {
            filtered = filtered.filter(k => k.jlpt_level > 0);
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
        
        case 'strokes':
          // Filter by stroke count range if specified
          if (activeStrokeTab !== 'all') {
            switch (activeStrokeTab) {
              case '1-5':
                filtered = filtered.filter(k => k.stroke_count >= 1 && k.stroke_count <= 5);
                break;
              case '6-10':
                filtered = filtered.filter(k => k.stroke_count >= 6 && k.stroke_count <= 10);
                break;
              case '11-20':
                filtered = filtered.filter(k => k.stroke_count >= 11 && k.stroke_count <= 20);
                break;
              case '20+':
                filtered = filtered.filter(k => k.stroke_count > 20);
                break;
            }
          }
          // Sort by stroke count, then by frequency
          filtered.sort((a, b) => {
            if (a.stroke_count !== b.stroke_count) {
              return a.stroke_count - b.stroke_count;
            }
            return (a.frequency || 9999) - (b.frequency || 9999);
          });
          break;
      }

      const processingTime = Date.now() - startTime;
      const minDisplayTime = 300; // Minimum time to show loading animation
      
      // Ensure loading animation shows for at least minDisplayTime
      const remainingTime = Math.max(0, minDisplayTime - processingTime);
      
      setTimeout(() => {
        filteredKanji = filtered;
        filteringLoading = false;
      }, remainingTime);
    }, 50); // Initial delay to show loading animation
  }

  function setActiveTab(tabId: string) {
    activeTab = tabId;
    if (tabId !== 'jlpt') {
      activeJlptTab = 'all';
    }
    if (tabId !== 'strokes') {
      activeStrokeTab = 'all';
    }
    filterKanji();
  }

  function setActiveJlptTab(tabId: string) {
    activeJlptTab = tabId;
    filterKanji();
  }

  function setActiveStrokeTab(tabId: string) {
    activeStrokeTab = tabId;
    filterKanji();
  }

  function handleSearch() {
    filterKanji();
  }

  function clearSearch() {
    searchQuery = '';
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
  
  <!-- Search bar -->
  <div class="search-container">
    <input
      type="text"
      bind:value={searchQuery}
      on:input={handleSearch}
      placeholder="Search by meaning, reading, or character..."
      class="search-input"
    />
    {#if searchQuery}
      <button class="clear-search-btn" on:click={clearSearch}>
        ✕
      </button>
    {/if}
  </div>
  
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

  <!-- Stroke count sub-tabs -->
  {#if activeTab === 'strokes'}
    <div class="sub-tab-container">
      {#each strokeTabs as tab}
        <button
          class="sub-tab {activeStrokeTab === tab.id ? 'active' : ''}"
          on:click={() => setActiveStrokeTab(tab.id)}
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
  {:else if filteringLoading}
    <div class="filtering-loading">
      <div class="spinner"></div>
      <span>Filtering kanji...</span>
    </div>
  {:else}
    <div class="kanji-stats">
      Showing {filteredKanji.length} kanji
      {#if searchQuery}
        for "{searchQuery}"
      {:else if activeTab === 'frequent'}
        (Top 1000 most frequent)
      {:else if activeTab === 'grade'}
        (Sorted by grade)
      {:else if activeTab === 'strokes'}
        {#if activeStrokeTab === 'all'}
          (Sorted by stroke count)
        {:else if activeStrokeTab === '1-5'}
          (1-5 strokes)
        {:else if activeStrokeTab === '6-10'}
          (6-10 strokes)
        {:else if activeStrokeTab === '11-20'}
          (11-20 strokes)
        {:else if activeStrokeTab === '20+'}
          (20+ strokes)
        {/if}
      {:else if activeTab === 'jlpt'}
        {#if activeJlptTab === 'all'}
          (All JLPT levels)
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
