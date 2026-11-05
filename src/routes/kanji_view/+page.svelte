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
  import { createKanjiSearch } from '$lib/search';
  import { type SrsCard, cardStage, STAGE_LABELS, formatDue } from '$lib/srs';
  import './kanji-view.css';

  // Cards are rendered in pages as the user scrolls; rendering all 13,000
  // kanji at once took several seconds.
  const PAGE_SIZE = 200;
  const SEARCH_DEBOUNCE_MS = 150;

  let kanjiList: Kanji[] = [];
  let search = createKanjiSearch([]);
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
  // Flashcard deck
  let deck = new Set<string>();
  let selectedCard: SrsCard | null = null;
  let cardBusy = false;
  let cardError = '';
  let confirmingRemove = false; // removing a reviewed card asks for a second click
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
  // Position of the open kanji in the current list, for previous/next
  $: selectedIndex = selectedKanji
    ? filteredKanji.findIndex((k) => k.character === selectedKanji?.character)
    : -1;

  const tabs = [
    { id: 'all', label: 'All Kanji' },
    { id: 'frequent', label: 'Top 1000 Frequent' },
    { id: 'grade', label: 'By Grade' },
    { id: 'jlpt', label: 'By JLPT Level' },
    { id: 'strokes', label: 'By Stroke Count' },
    { id: 'deck', label: 'My Flashcards' }
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
    // Links from other pages: ?tab=deck, ?q=water, ?kanji=水
    const params = new URLSearchParams(window.location.search);
    const tab = params.get('tab');
    if (tab && tabs.some((t) => t.id === tab)) activeTab = tab;
    searchQuery = params.get('q') ?? '';

    await Promise.all([loadKanji(), loadDeck()]);

    const kanjiParam = params.get('kanji');
    const linked = kanjiParam ? kanjiList.find((k) => k.character === kanjiParam) : undefined;
    if (linked) openKanjiModal(linked);
  });

  async function loadDeck() {
    try {
      const characters: string[] = await invoke('get_deck_characters');
      deck = new Set(characters);
      if (activeTab === 'deck') filterKanji();
    } catch (err) {
      console.error('Failed to load flashcard deck:', err);
    }
  }

  onDestroy(() => {
    clearTimeout(searchTimer);
    stopAnimation();
  });

  async function loadKanji() {
    try {
      loading = true;
      error = '';
      kanjiList = await invoke('get_all_kanji');
      search = createKanjiSearch(kanjiList);
      filterKanji();
    } catch (err) {
      error = `Failed to load kanji: ${err}`;
      console.error('Error loading kanji:', err);
    } finally {
      loading = false;
    }
  }

  const byFrequency = (a: Kanji, b: Kanji) => (a.frequency || 9999) - (b.frequency || 9999);

  function filterKanji() {
    // Search results come back best match first; the tabs below re-sort
    // them by their own criteria.
    appliedQuery = searchQuery.trim();
    let filtered = appliedQuery ? search(appliedQuery) : kanjiList;

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

      case 'deck':
        filtered = filtered.filter((k) => deck.has(k.character));
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
    cardError = '';
    confirmingRemove = false;
    modalOpen = true;
    svgLoading = true;
    kanjiSvg = '';
    totalStrokes = 0;
    currentStroke = 0;
    selectedCard = null;

    try {
      const [details, card] = await Promise.all([
        invoke<KanjiLookup>('get_kanji', { character: kanji.character }),
        invoke<SrsCard | null>('get_card', { character: kanji.character }).catch((err) => {
          console.error('Failed to load flashcard:', err);
          return null;
        })
      ]);
      if (request !== modalRequest) return;
      selectedKanji = details;
      selectedCard = card;
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

  async function addToDeck() {
    if (!selectedKanji || cardBusy) return;
    const character = selectedKanji.character;
    cardBusy = true;
    cardError = '';
    try {
      const card: SrsCard = await invoke('add_card', { character });
      if (selectedKanji?.character === character) selectedCard = card;
      deck = new Set(deck).add(character);
      if (activeTab === 'deck') filterKanji();
    } catch (err) {
      cardError = `${err}`;
    } finally {
      cardBusy = false;
    }
  }

  async function removeFromDeck() {
    if (!selectedKanji || cardBusy) return;
    const character = selectedKanji.character;
    // window.confirm isn't available in every Tauri webview, so a card with
    // review history needs a second click instead.
    const hasProgress = selectedCard !== null && selectedCard.total_reviews > 0;
    if (hasProgress && !confirmingRemove) {
      confirmingRemove = true;
      return;
    }
    confirmingRemove = false;
    cardBusy = true;
    cardError = '';
    try {
      await invoke('remove_card', { character });
      if (selectedKanji?.character === character) selectedCard = null;
      const next = new Set(deck);
      next.delete(character);
      deck = next;
      if (activeTab === 'deck') filterKanji();
    } catch (err) {
      cardError = `${err}`;
    } finally {
      cardBusy = false;
    }
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
    if (!modalOpen) return;
    if (event.key === 'Escape') {
      closeModal();
      return;
    }
    // Arrow keys step through the list, unless a control (like the speed
    // slider) is using them
    const target = event.target as HTMLElement | null;
    if (target?.closest('input, select, textarea')) return;
    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      event.preventDefault();
      showAdjacent(event.key === 'ArrowLeft' ? -1 : 1);
    }
  }

  function showAdjacent(offset: number) {
    if (selectedIndex < 0) return;
    const index = selectedIndex + offset;
    const next = filteredKanji[index];
    if (!next) return;
    // Keep the grid behind the dialog in step with the dialog
    if (index >= visibleCount) visibleCount = index + 1;
    openKanjiModal(next);
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
      placeholder="Search by meaning, reading (kana or romaji), or kanji..."
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
        {tab.label}{tab.id === 'deck' ? ` (${deck.size})` : ''}
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
      {:else if activeTab === 'deck'}
        (in your flashcards)
      {/if}
    </div>

    {#if activeTab === 'deck' && deck.size === 0}
      <div class="empty-state">
        Your flashcard deck is empty. Open any kanji and choose
        <strong>Add to flashcards</strong>, or add a batch from the
        <a href="/flashcards">Flashcards</a> page.
      </div>
    {/if}

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
              {#if deck.has(kanji.character)}
                <span class="meta-item deck" title="In your flashcards">★ Flashcard</span>
              {/if}
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
        <div class="modal-header-actions">
          {#if selectedIndex >= 0}
            <button
              class="nav-arrow"
              on:click={() => showAdjacent(-1)}
              disabled={selectedIndex === 0}
              aria-label="Previous kanji"
              title="Previous (←)"
            >‹</button>
            <span class="nav-position">{selectedIndex + 1} / {filteredKanji.length}</span>
            <button
              class="nav-arrow"
              on:click={() => showAdjacent(1)}
              disabled={selectedIndex === filteredKanji.length - 1}
              aria-label="Next kanji"
              title="Next (→)"
            >›</button>
          {/if}
          <button class="close-button" on:click={closeModal} aria-label="Close modal">×</button>
        </div>
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
            <h3>Flashcards</h3>
            {#if selectedCard}
              <div class="modal-reading">
                <span class="modal-label">Status:</span>
                {STAGE_LABELS[cardStage(selectedCard)]}
                {#if selectedCard.total_reviews > 0}
                  · {formatDue(selectedCard.next_review)}
                  · {selectedCard.correct_reviews}/{selectedCard.total_reviews} correct
                {/if}
              </div>
              <button
                class="deck-button remove {confirmingRemove ? 'confirming' : ''}"
                on:click={removeFromDeck}
                disabled={cardBusy}
              >
                {confirmingRemove ? 'Click again to remove (progress will be lost)' : 'Remove from flashcards'}
              </button>
            {:else}
              <button class="deck-button add" on:click={addToDeck} disabled={cardBusy}>
                ＋ Add to flashcards
              </button>
            {/if}
            {#if cardError}
              <div class="card-error">{cardError}</div>
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
