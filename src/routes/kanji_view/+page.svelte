<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import {
    type Kanji,
    type KanjiLookup,
    splitMeanings,
    getGradeLabel,
    getJlptLabel,
    JLPT_LEVEL_HELP
  } from '$lib/kanji';
  import './kanji-view.css';

  // Cards are rendered in pages as the user scrolls; rendering all 13,000
  // kanji at once took several seconds.
  const PAGE_SIZE = 200;
  const SEARCH_DEBOUNCE_MS = 150;

  let kanjiList: Kanji[] = [];
  let filteredKanji: Kanji[] = [];
  let visibleCount = PAGE_SIZE;
  let loading = true;
  let error = '';
  let detailError = '';
  let activeTab = 'all';
  let activeJlptTab = 'all';
  let activeStrokeTab = 'all';
  let searchQuery = '';
  let appliedQuery = ''; // the query the current results were filtered with
  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  let selectedKanji: KanjiLookup | null = null;
  let modalOpen = false;
  let modalRequest = 0; // increments on every open/close so stale loads are ignored
  let kanjiSvg = '';
  let svgLoading = false;

  // Stroke order animation
  let isAnimating = false;
  let animationSpeed = 1000; // milliseconds per stroke
  let currentStroke = 0;
  let totalStrokes = 0;
  let animationRun = 0; // increments to cancel a running animation
  let animationTimer: ReturnType<typeof setTimeout> | null = null;

  $: visibleKanji = filteredKanji.slice(0, visibleCount);

  const tabs = [
    { id: 'all', label: 'All Kanji' },
    { id: 'frequent', label: 'Top 1000 Frequent' },
    { id: 'grade', label: 'By Grade' },
    { id: 'jlpt', label: 'By JLPT Level' },
    { id: 'strokes', label: 'By Stroke Count' }
  ];

  const jlptTabs = [
    { id: 'all', label: 'All JLPT' },
    ...[4, 3, 2, 1].map((level) => ({ id: String(level), label: getJlptLabel(level) }))
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

  onDestroy(() => {
    clearTimeout(searchTimer);
    stopAnimation();
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

  function matchesSearch(kanji: Kanji, query: string): boolean {
    return (
      kanji.character.includes(query) ||
      kanji.meanings.toLowerCase().includes(query) ||
      kanji.onyomi.toLowerCase().includes(query) ||
      kanji.kunyomi.toLowerCase().includes(query)
    );
  }

  const byFrequency = (a: Kanji, b: Kanji) => (a.frequency || 9999) - (b.frequency || 9999);

  function filterKanji() {
    let filtered = kanjiList;

    const query = searchQuery.toLowerCase().trim();
    appliedQuery = searchQuery.trim();
    if (query) {
      filtered = filtered.filter((kanji) => matchesSearch(kanji, query));
    }

    switch (activeTab) {
      case 'frequent':
        // Frequency 0 means the kanji isn't ranked
        filtered = filtered
          .filter((k) => k.frequency > 0)
          .sort((a, b) => a.frequency - b.frequency)
          .slice(0, 1000);
        break;

      case 'grade':
        filtered = filtered.filter((k) => k.grade > 0).sort((a, b) => a.grade - b.grade);
        break;

      case 'jlpt':
        if (activeJlptTab === 'all') {
          filtered = filtered.filter((k) => k.jlpt_level > 0);
        } else {
          const level = parseInt(activeJlptTab);
          filtered = filtered.filter((k) => k.jlpt_level === level);
        }
        // Easiest level first, then by frequency
        filtered = [...filtered].sort(
          (a, b) => b.jlpt_level - a.jlpt_level || byFrequency(a, b)
        );
        break;

      case 'strokes': {
        const ranges: Record<string, [number, number]> = {
          '1-5': [1, 5],
          '6-10': [6, 10],
          '11-20': [11, 20],
          '20+': [21, Infinity]
        };
        const range = ranges[activeStrokeTab];
        if (range) {
          filtered = filtered.filter(
            (k) => k.stroke_count >= range[0] && k.stroke_count <= range[1]
          );
        }
        filtered = [...filtered].sort(
          (a, b) => a.stroke_count - b.stroke_count || byFrequency(a, b)
        );
        break;
      }
    }

    filteredKanji = filtered;
    visibleCount = PAGE_SIZE;
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
    clearTimeout(searchTimer);
    searchTimer = setTimeout(filterKanji, SEARCH_DEBOUNCE_MS);
  }

  function clearSearch() {
    clearTimeout(searchTimer);
    searchQuery = '';
    filterKanji();
  }

  function showMore() {
    visibleCount = Math.min(visibleCount + PAGE_SIZE, filteredKanji.length);
  }

  // Loads the next page when the sentinel below the grid scrolls into view.
  function infiniteScroll(node: HTMLElement) {
    const observer = new IntersectionObserver(
      async (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        showMore();
        await tick();
        // Re-observing reports the current state again, so this keeps
        // loading while the sentinel is still on screen (e.g. tall windows).
        observer.unobserve(node);
        observer.observe(node);
      },
      { rootMargin: '800px' }
    );
    observer.observe(node);
    return {
      destroy() {
        observer.disconnect();
      }
    };
  }

  function goToHome() {
    goto('/');
  }

  // First three meanings for the grid cards
  function formatMeanings(meanings: string): string {
    return splitMeanings(meanings).slice(0, 3).join('; ');
  }

  async function openKanjiModal(kanji: Kanji) {
    const request = ++modalRequest;
    stopAnimation();
    detailError = '';
    modalOpen = true;
    svgLoading = true;
    kanjiSvg = '';
    totalStrokes = 0;
    currentStroke = 0;

    try {
      const details: KanjiLookup = await invoke('get_kanji', { character: kanji.character });
      if (request !== modalRequest) return;
      selectedKanji = details;
    } catch (err) {
      if (request !== modalRequest) return;
      console.error('Error loading kanji details:', err);
      modalOpen = false;
      svgLoading = false;
      detailError = `Could not load details for ${kanji.character}: ${err}`;
      return;
    }

    let svg = '';
    try {
      svg = await invoke('get_kanji_svg', { character: kanji.character });
    } catch (svgError) {
      console.warn('No stroke order diagram for', kanji.character, svgError);
    }
    if (request !== modalRequest) return;

    kanjiSvg = svg;
    svgLoading = false;
    if (!svg) return;

    await tick();
    if (request !== modalRequest) return;
    if (isAnimCJKSource(svg)) {
      // AnimCJK diagrams are injected directly; KanjiVG ones are rendered
      // by the template with {@html}.
      const wrapper = document.querySelector('.svg-wrapper');
      if (wrapper) wrapper.innerHTML = svg;
    }
    prepareDiagram();
  }

  function closeModal() {
    modalRequest++;
    stopAnimation();
    modalOpen = false;
    selectedKanji = null;
    kanjiSvg = '';
    svgLoading = false;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && modalOpen) {
      closeModal();
    }
  }

  // Close only when the backdrop itself is clicked, not the dialog
  function handleOverlayClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeModal();
    }
  }

  function handleCardKeydown(event: KeyboardEvent, kanji: Kanji) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      openKanjiModal(kanji);
    }
  }

  // ---- Stroke order diagram ----

  function isAnimCJKSource(svg: string): boolean {
    return svg.includes('class="acjk"');
  }

  function getDiagram(): SVGSVGElement | null {
    return document.querySelector('.svg-wrapper svg');
  }

  function isAnimCJK(svg: SVGSVGElement): boolean {
    return svg.classList.contains('acjk');
  }

  // Stroke paths in drawing order
  function getStrokePaths(svg: SVGSVGElement): SVGPathElement[] {
    if (isAnimCJK(svg)) {
      return Array.from(svg.querySelectorAll<SVGPathElement>('path[clip-path]'));
    }
    // KanjiVG stroke ids look like "kvg:065e5-s1"
    let paths = svg.querySelectorAll<SVGPathElement>('path[id*="-s"]');
    if (paths.length === 0) paths = svg.querySelectorAll<SVGPathElement>('g[id^="kvg:"] path');
    if (paths.length === 0) paths = svg.querySelectorAll<SVGPathElement>('path');
    return Array.from(paths);
  }

  // KanjiVG stroke number labels, in stroke order
  function getStrokeNumbers(svg: SVGSVGElement): SVGTextElement[] {
    if (isAnimCJK(svg)) return [];
    return Array.from(svg.querySelectorAll<SVGTextElement>('g[id*="StrokeNumbers"] text'));
  }

  function setStrokeVisible(
    svg: SVGSVGElement,
    path: SVGPathElement,
    visible: boolean,
    animate: boolean
  ) {
    if (isAnimCJK(svg)) {
      path.style.transition = animate
        ? `stroke-dashoffset ${animationSpeed / 1000}s ease-out`
        : 'none';
      path.style.strokeDashoffset = visible ? '0' : '3339';
    } else {
      path.style.transition = animate ? 'opacity 0.3s ease' : 'none';
      path.style.opacity = visible ? '1' : '0';
    }
  }

  function prepareDiagram() {
    const svg = getDiagram();
    if (!svg) return;

    // Size is set in kanji-view.css
    if (isAnimCJK(svg)) {
      svg.style.backgroundColor = 'white';
      // Light grey outline of the whole character behind the strokes
      svg.querySelectorAll<SVGPathElement>('path[id]').forEach((path) => {
        Object.assign(path.style, {
          stroke: '#ddd',
          strokeWidth: '22px',
          fill: 'none',
          opacity: '0.4',
          strokeLinecap: 'round',
          strokeLinejoin: 'round'
        });
      });
      getStrokePaths(svg).forEach((path) => {
        Object.assign(path.style, {
          stroke: '#000',
          strokeWidth: '80px',
          fill: 'none',
          opacity: '1',
          strokeLinecap: 'round',
          strokeLinejoin: 'round',
          strokeDasharray: '3339',
          animation: 'none' // replace the SVG's own autoplay animation
        });
      });
    }

    totalStrokes = getStrokePaths(svg).length;
    showAllStrokes();
  }

  function startAnimation() {
    const svg = getDiagram();
    if (!svg || totalStrokes === 0) return;

    stopAnimation();
    const run = animationRun;
    const strokes = getStrokePaths(svg);
    const numbers = getStrokeNumbers(svg);
    strokes.forEach((path) => setStrokeVisible(svg, path, false, false));
    numbers.forEach((label) => (label.style.opacity = '0'));
    currentStroke = 0;
    isAnimating = true;

    // Each step reads animationSpeed, so moving the slider mid-animation
    // just changes the pace instead of starting another animation.
    const step = () => {
      if (run !== animationRun) return;
      if (currentStroke >= strokes.length) {
        animationTimer = null;
        isAnimating = false;
        return;
      }
      setStrokeVisible(svg, strokes[currentStroke], true, true);
      if (numbers[currentStroke]) numbers[currentStroke].style.opacity = '1';
      currentStroke += 1;
      animationTimer = setTimeout(step, animationSpeed);
    };
    animationTimer = setTimeout(step, 100);
  }

  function stopAnimation() {
    animationRun++;
    if (animationTimer) {
      clearTimeout(animationTimer);
      animationTimer = null;
    }
    isAnimating = false;
  }

  function resetAnimation() {
    const svg = getDiagram();
    if (!svg) return;
    stopAnimation();
    getStrokePaths(svg).forEach((path) => setStrokeVisible(svg, path, false, false));
    getStrokeNumbers(svg).forEach((label) => (label.style.opacity = '0'));
    currentStroke = 0;
  }

  function showAllStrokes() {
    const svg = getDiagram();
    if (!svg) return;
    stopAnimation();
    getStrokePaths(svg).forEach((path) => setStrokeVisible(svg, path, true, false));
    getStrokeNumbers(svg).forEach((label) => (label.style.opacity = '1'));
    currentStroke = totalStrokes;
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
      <button class="clear-search-btn" on:click={clearSearch} aria-label="Clear search">
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
          title={tab.id === 'all' ? undefined : JLPT_LEVEL_HELP}
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

  {#if detailError}
    <div class="error detail-error">
      {detailError}
      <button class="dismiss-button" on:click={() => (detailError = '')} aria-label="Dismiss">✕</button>
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
      {#if appliedQuery}
        for "{appliedQuery}"
      {:else if activeTab === 'frequent'}
        (Top 1000 most frequent)
      {:else if activeTab === 'grade'}
        (Sorted by grade)
      {:else if activeTab === 'strokes'}
        {#if activeStrokeTab === 'all'}
          (Sorted by stroke count)
        {:else}
          ({activeStrokeTab} strokes)
        {/if}
      {:else if activeTab === 'jlpt'}
        {#if activeJlptTab === 'all'}
          (All JLPT levels)
        {:else}
          ({getJlptLabel(parseInt(activeJlptTab))})
        {/if}
      {/if}
    </div>

    <div class="kanji-grid">
      {#each visibleKanji as kanji (kanji.character)}
        <div
          class="kanji-card"
          on:click={() => openKanjiModal(kanji)}
          on:keydown={(e) => handleCardKeydown(e, kanji)}
          tabindex="0"
          role="button"
        >
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
                <span class="meta-item jlpt" title={JLPT_LEVEL_HELP}>{getJlptLabel(kanji.jlpt_level)}</span>
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

    {#if visibleCount < filteredKanji.length}
      <div class="load-more" use:infiniteScroll>
        <button class="load-more-button" on:click={showMore}>
          Show more ({filteredKanji.length - visibleCount} remaining)
        </button>
      </div>
    {/if}
  {/if}
</main>

<!-- Kanji Details Modal -->
{#if modalOpen && selectedKanji}
  <!-- Escape is handled by the window listener below. The dialog content
       must not stop keydown propagation, or Escape stops working once a
       control inside the dialog has focus. -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="modal-overlay" on:click={handleOverlayClick} tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-content" role="document">
      <div class="modal-header">
        <h2 id="modal-title">Kanji Details</h2>
        <button class="close-button" on:click={closeModal} aria-label="Close modal">×</button>
      </div>

      <div class="modal-body">
        <div class="modal-left">
          <div class="modal-kanji-character">{selectedKanji.character}</div>

          <div class="modal-svg-container">
            {#if svgLoading}
              <div class="svg-loading">Loading stroke order...</div>
            {:else if kanjiSvg}
              <div class="svg-wrapper">
                <!-- AnimCJK SVGs are injected via JavaScript, KanjiVG SVGs use {@html} -->
                {#if !isAnimCJKSource(kanjiSvg)}
                  {@html kanjiSvg}
                {/if}
              </div>
              <!-- Animation Controls -->
              <div class="animation-controls">
                <div class="animation-buttons">
                  <button class="control-btn" on:click={startAnimation} disabled={isAnimating || totalStrokes === 0}>
                    {isAnimating ? 'Playing...' : 'Play'}
                  </button>
                  <button class="control-btn" on:click={stopAnimation} disabled={!isAnimating}>
                    Stop
                  </button>
                  <button class="control-btn" on:click={resetAnimation} disabled={totalStrokes === 0}>
                    Reset
                  </button>
                  <button class="control-btn" on:click={showAllStrokes} disabled={totalStrokes === 0}>
                    Show All
                  </button>
                </div>

                <div class="animation-info">
                  <span class="stroke-counter">
                    Stroke {currentStroke} of {totalStrokes}
                  </span>
                </div>

                <div class="speed-control">
                  <label for="speed-slider">Speed:</label>
                  <input
                    id="speed-slider"
                    type="range"
                    min="200"
                    max="2000"
                    step="100"
                    bind:value={animationSpeed}
                  />
                  <span class="speed-display">{(2200 - animationSpeed) / 1000}x</span>
                </div>
              </div>
            {:else}
              <div class="svg-placeholder">No stroke order available</div>
            {/if}
          </div>
        </div>

        <div class="modal-right">
          <div class="modal-info-section">
            <h3>Readings</h3>
            {#if selectedKanji.onyomi}
              <div class="modal-reading">
                <span class="modal-label">On'yomi:</span> {selectedKanji.onyomi}
              </div>
            {/if}
            {#if selectedKanji.kunyomi}
              <div class="modal-reading">
                <span class="modal-label">Kun'yomi:</span> {selectedKanji.kunyomi}
              </div>
            {/if}
            {#if selectedKanji.nanori && selectedKanji.nanori.length > 0}
              <div class="modal-reading">
                <span class="modal-label">Nanori:</span> {selectedKanji.nanori.join(', ')}
              </div>
            {/if}
          </div>

          <div class="modal-info-section">
            <h3>Meanings</h3>
            {#if selectedKanji.meanings && selectedKanji.meanings.length > 0}
              <div class="modal-meanings">
                {#each selectedKanji.meanings as meaning}
                  <span class="meaning-tag">{meaning}</span>
                {/each}
              </div>
            {:else}
              <div class="no-meanings">No meanings available</div>
            {/if}
          </div>

          <div class="modal-info-section">
            <h3>Properties</h3>
            <div class="modal-meta">
              <div class="modal-meta-item">
                <span class="modal-label">Stroke Count:</span> {selectedKanji.stroke_count}
              </div>
              {#if selectedKanji.grade > 0}
                <div class="modal-meta-item">
                  <span class="modal-label">Grade:</span> {getGradeLabel(selectedKanji.grade)}
                </div>
              {/if}
              {#if selectedKanji.jlpt_level > 0}
                <div class="modal-meta-item" title={JLPT_LEVEL_HELP}>
                  <span class="modal-label">JLPT Level:</span> {getJlptLabel(selectedKanji.jlpt_level)}
                </div>
              {/if}
              {#if selectedKanji.frequency > 0}
                <div class="modal-meta-item">
                  <span class="modal-label">Frequency Rank:</span> #{selectedKanji.frequency}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<svelte:window on:keydown={handleKeydown} />
