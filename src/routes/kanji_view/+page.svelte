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

  interface KanjiLookup {
    character: string;
    stroke_count: number;
    grade: number;
    jlpt_level: number;
    frequency: number;
    onyomi: string;
    kunyomi: string;
    meanings: string[];
    nanori: string[];
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
  let selectedKanji: KanjiLookup | null = null;
  let modalOpen = false;
  let kanjiSvg = '';
  let svgLoading = false;
  
  // Animation state variables
  let isAnimating = false;
  let animationSpeed = 1000; // milliseconds per stroke
  let currentStroke = 0;
  let totalStrokes = 0;
  let animationInterval: ReturnType<typeof setInterval> | null = null;

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

  async function openKanjiModal(kanji: Kanji) {
    try {
      modalOpen = true;
      svgLoading = true;
      kanjiSvg = '';
      
      // Get detailed kanji information
      selectedKanji = await invoke('get_kanji', { character: kanji.character });
      
      // Get kanji SVG
      try {
        kanjiSvg = await invoke('get_kanji_svg', { character: kanji.character });
        
        console.log('Received SVG from backend:', kanjiSvg ? 'SVG content received' : 'No SVG content');
        console.log('SVG length:', kanjiSvg ? kanjiSvg.length : 0);
        console.log('SVG starts with:', kanjiSvg ? kanjiSvg.substring(0, 100) : 'N/A');
        console.log('SVG contains acjk class:', kanjiSvg ? kanjiSvg.includes('class="acjk"') : false);
        
        // Initialize animation after SVG loads
        if (kanjiSvg) {
          // Use a small delay to ensure DOM is updated
          setTimeout(() => {
            console.log('About to initialize animation...');
            injectAndInitializeAnimation();
          }, 100);
        }
      } catch (svgError) {
        console.warn('SVG not found for', kanji.character, svgError);
        kanjiSvg = '';
      }
      
      svgLoading = false;
    } catch (error) {
      console.error('Error loading kanji details:', error);
      svgLoading = false;
    }
  }

  function closeModal() {
    modalOpen = false;
    selectedKanji = null;
    kanjiSvg = '';
    stopAnimation();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && modalOpen) {
      closeModal();
    }
  }

  // SVG injection and animation initialization
  function injectAndInitializeAnimation() {
    if (!kanjiSvg) {
      console.log('No kanjiSvg content available');
      return;
    }
    
    console.log('InjectAndInitializeAnimation called, kanjiSvg length:', kanjiSvg.length);
    
    // Wait for DOM to be ready
    const svgWrapper = document.querySelector('.svg-wrapper');
    
    if (!svgWrapper) {
      console.warn('SVG wrapper not found');
      return;
    }
    
    // Check if this is an AnimCJK SVG by looking for the acjk class in the SVG content
    const isAnimCJK = kanjiSvg.includes('class="acjk"');
    
    console.log('Is AnimCJK SVG:', isAnimCJK);
    
    if (isAnimCJK) {
      // AnimCJK SVG - use fetch/innerHTML method as recommended
      console.log('AnimCJK SVG detected - using direct innerHTML injection');
      
      // Clear the wrapper and inject the SVG directly
      svgWrapper.innerHTML = kanjiSvg;
      
      console.log('SVG injected, wrapper innerHTML length:', svgWrapper.innerHTML.length);
      
      // Force style override immediately after injection
      const injectedSvg = svgWrapper.querySelector('svg');
      if (injectedSvg) {
        // Force SVG dimensions and visibility
        injectedSvg.style.width = '300px';
        injectedSvg.style.height = '300px';
        injectedSvg.style.display = 'block';
        injectedSvg.style.backgroundColor = 'white';
        
        console.log('SVG styled immediately after injection');
      }
      
      // Initialize AnimCJK animation
      const svgContainer = svgWrapper.querySelector('svg');
      if (svgContainer) {
        console.log('AnimCJK SVG successfully injected');
        console.log('SVG element:', svgContainer);
        console.log('SVG classes:', svgContainer.classList);
        console.log('SVG computed style width:', window.getComputedStyle(svgContainer).width);
        console.log('SVG computed style height:', window.getComputedStyle(svgContainer).height);
        console.log('SVG bounding box:', svgContainer.getBoundingClientRect());
        
        // Find stroke paths with clip-path (animated paths)
        const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
        const staticPaths = svgContainer.querySelectorAll('path[id]');
        totalStrokes = strokePaths.length;
        currentStroke = 0;
        
        console.log(`Found ${totalStrokes} animated strokes and ${staticPaths.length} static paths for AnimCJK kanji`);
        
        // Debug stroke paths styling
        strokePaths.forEach((path, index) => {
          const pathElement = path as HTMLElement;
          const computedStyle = window.getComputedStyle(pathElement);
          console.log(`Animated stroke ${index + 1}:`, {
            stroke: computedStyle.stroke,
            strokeWidth: computedStyle.strokeWidth,
            strokeDasharray: computedStyle.strokeDasharray,
            strokeDashoffset: computedStyle.strokeDashoffset,
            fill: computedStyle.fill,
            opacity: computedStyle.opacity
          });
        });
        
        // Debug static paths styling
        staticPaths.forEach((path, index) => {
          const pathElement = path as HTMLElement;
          const computedStyle = window.getComputedStyle(pathElement);
          console.log(`Static stroke ${index + 1}:`, {
            fill: computedStyle.fill,
            stroke: computedStyle.stroke,
            strokeWidth: computedStyle.strokeWidth,
            opacity: computedStyle.opacity
          });
        });
        
        // Show all strokes immediately when modal opens for now
        // Later we can change this to start with hidden strokes
        setTimeout(() => {
          showAllAnimCJKStrokes();
        }, 100);
      } else {
        console.error('Failed to find injected SVG element');
      }
    } else {
      // KanjiVG SVG - use Svelte's reactive binding (kanjiSvg will be used in template)
      console.log('KanjiVG SVG detected - using Svelte template binding');
      
      // Clear the wrapper (let Svelte handle it via {@html})
      svgWrapper.innerHTML = '';
      
      // Initialize KanjiVG animation after DOM updates
      setTimeout(() => {
        initializeKanjiVGAnimation();
      }, 50);
    }
  }

  // Animation control functions
  function initializeKanjiVGAnimation() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    
    console.log('SVG container element:', svgContainer);
    
    if (!svgContainer) {
      console.warn('SVG container not found for KanjiVG');
      return;
    }
    
    console.log('KanjiVG SVG detected - using manual animation');
    
    // Find stroke paths - KanjiVG format uses path elements with kvg:*-s* IDs
    let strokePaths = svgContainer.querySelectorAll('path[id*="-s"]');
    
    // If no stroke paths found, try alternative patterns
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('g[id^="kvg:"] path');
    }
    
    // If still no strokes found, try to find all path elements
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('path');
    }
    
    totalStrokes = strokePaths.length;
    currentStroke = 0;
    
    console.log(`Found ${totalStrokes} strokes for KanjiVG kanji`);
    
    // Reset animation state
    resetKanjiVGAnimation();
    
    // Initialize all strokes as visible by default
    showAllStrokes();
  }

  function startAnimation() {
    if (isAnimating || totalStrokes === 0) return;
    
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    // Check if this is an AnimCJK SVG
    const isAnimCJK = svgContainer.classList.contains('acjk');
    
    if (isAnimCJK) {
      startAnimCJKAnimation();
    } else {
      // KanjiVG animation logic
      isAnimating = true;
      currentStroke = 0;
      
      // Hide all strokes initially
      hideAllKanjiVGStrokes();
      
      // Start animation interval
      animationInterval = setInterval(() => {
        if (currentStroke < totalStrokes) {
          showKanjiVGStroke(currentStroke);
          currentStroke++;
        } else {
          stopAnimation();
        }
      }, animationSpeed);
    }
  }

  function stopAnimation() {
    if (animationInterval) {
      clearInterval(animationInterval);
      animationInterval = null;
    }
    isAnimating = false;
  }

  function resetAnimation() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    // Check if this is an AnimCJK SVG
    const isAnimCJK = svgContainer.classList.contains('acjk');
    
    if (isAnimCJK) {
      resetAnimCJKAnimation();
    } else {
      // KanjiVG reset logic
      resetKanjiVGAnimation();
    }
  }

  function resetKanjiVGAnimation() {
    stopAnimation();
    currentStroke = 0;
    hideAllKanjiVGStrokes();
  }

  function hideAllKanjiVGStrokes() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    // Find stroke paths - KanjiVG format
    let strokePaths = svgContainer.querySelectorAll('path[id*="-s"]');
    
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('g[id^="kvg:"] path');
    }
    
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('path');
    }
    
    strokePaths.forEach((stroke) => {
      (stroke as HTMLElement).style.opacity = '0';
      (stroke as HTMLElement).style.transition = 'opacity 0.3s ease';
    });
  }

  function showKanjiVGStroke(strokeIndex: number) {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    // Find stroke paths - KanjiVG format
    let strokePaths = svgContainer.querySelectorAll('path[id*="-s"]');
    
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('g[id^="kvg:"] path');
    }
    
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('path');
    }
    
    const strokePath = strokePaths[strokeIndex];
    if (strokePath) {
      (strokePath as HTMLElement).style.opacity = '1';
      (strokePath as HTMLElement).style.transition = 'opacity 0.3s ease';
    }
  }

  function showAllKanjiVGStrokes() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    // Find stroke paths - KanjiVG format
    let strokePaths = svgContainer.querySelectorAll('path[id*="-s"]');
    
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('g[id^="kvg:"] path');
    }
    
    if (strokePaths.length === 0) {
      strokePaths = svgContainer.querySelectorAll('path');
    }
    
    strokePaths.forEach((stroke) => {
      (stroke as HTMLElement).style.opacity = '1';
      (stroke as HTMLElement).style.transition = 'opacity 0.3s ease';
    });
    currentStroke = totalStrokes;
  }

  function showAllStrokes() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    // Check if this is an AnimCJK SVG
    const isAnimCJK = svgContainer.classList.contains('acjk');
    
    if (isAnimCJK) {
      showAllAnimCJKStrokes();
    } else {
      // KanjiVG show all logic
      showAllKanjiVGStrokes();
    }
  }

  function changeAnimationSpeed(newSpeed: number) {
    animationSpeed = newSpeed;
    
    // If currently animating, restart with new speed
    if (isAnimating) {
      const svgContainer = document.querySelector('.svg-wrapper svg');
      if (svgContainer) {
        const isAnimCJK = svgContainer.classList.contains('acjk');
        
        if (isAnimCJK) {
          // For AnimCJK, we need to restart the animation
          resetAnimCJKAnimation();
          setTimeout(() => startAnimCJKAnimation(), 100);
        } else {
          // For KanjiVG, restart the interval-based animation
          stopAnimation();
          startAnimation();
        }
      }
    }
  }

  // AnimCJK animation functions
  function resetAnimCJKAnimation() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    console.log('Resetting AnimCJK animation');
    
    // Stop any running animation
    isAnimating = false;
    currentStroke = 0;
    
    // Get all animated paths and hide them
    const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
    strokePaths.forEach((path) => {
      const pathElement = path as HTMLElement;
      
      // Set up stroke styling
      pathElement.style.stroke = '#000';
      pathElement.style.strokeWidth = '80px';
      pathElement.style.fill = 'none';
      pathElement.style.opacity = '1';
      pathElement.style.strokeLinecap = 'round';
      pathElement.style.strokeLinejoin = 'round';
      pathElement.style.strokeDasharray = '3339';
      pathElement.style.strokeDashoffset = '3339'; // Hide all strokes
      pathElement.style.transition = 'none';
    });
    
    // Also style static paths
    const staticPaths = svgContainer.querySelectorAll('path[id]');
    staticPaths.forEach((path) => {
      const pathElement = path as HTMLElement;
      pathElement.style.stroke = '#ddd';
      pathElement.style.strokeWidth = '22px';
      pathElement.style.fill = 'none';
      pathElement.style.opacity = '0.4';
      pathElement.style.strokeLinecap = 'round';
      pathElement.style.strokeLinejoin = 'round';
    });
    
    console.log('AnimCJK animation reset complete');
  }

  function startAnimCJKAnimation() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    isAnimating = true;
    currentStroke = 0;
    
    // Get all animated paths
    const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
    
    console.log('Starting AnimCJK animation with', strokePaths.length, 'strokes');
    
    // First, hide all strokes by setting stroke-dashoffset to a large value
    strokePaths.forEach((path, index) => {
      const pathElement = path as HTMLElement;
      
      // Set up stroke styling for animation
      pathElement.style.stroke = '#000';
      pathElement.style.strokeWidth = '80px';
      pathElement.style.fill = 'none';
      pathElement.style.opacity = '1';
      pathElement.style.strokeLinecap = 'round';
      pathElement.style.strokeLinejoin = 'round';
      pathElement.style.strokeDasharray = '3339';
      pathElement.style.strokeDashoffset = '3339'; // Hide initially
      pathElement.style.transition = 'none'; // Remove any existing transitions
    });
    
    // Animate strokes one by one
    let currentAnimatingStroke = 0;
    
    const animateNextStroke = () => {
      if (currentAnimatingStroke < strokePaths.length && isAnimating) {
        const pathElement = strokePaths[currentAnimatingStroke] as HTMLElement;
        
        console.log(`Animating stroke ${currentAnimatingStroke + 1}`);
        
        // Set transition for smooth animation
        pathElement.style.transition = `stroke-dashoffset ${animationSpeed / 1000}s ease-out`;
        
        // Animate to visible (dashoffset 0)
        pathElement.style.strokeDashoffset = '0';
        
        currentStroke = currentAnimatingStroke + 1;
        currentAnimatingStroke++;
        
        // Schedule next stroke
        setTimeout(animateNextStroke, animationSpeed);
      } else {
        // Animation complete
        isAnimating = false;
        currentStroke = totalStrokes;
        console.log('AnimCJK animation complete');
      }
    };
    
    // Start the animation
    setTimeout(animateNextStroke, 100);
  }

  function showAllAnimCJKStrokes() {
    const svgContainer = document.querySelector('.svg-wrapper svg');
    if (!svgContainer) return;
    
    // For AnimCJK, showing all strokes means making them visible immediately
    // The AnimCJK animation works by animating stroke-dashoffset from 3339 to 0
    // To show all strokes, we set stroke-dashoffset to 0 for all paths
    const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
    console.log('Found', strokePaths.length, 'animated stroke paths');
    
    strokePaths.forEach((path, index) => {
      const pathElement = path as HTMLElement;
      
      // Log current state before modification
      const currentDashOffset = window.getComputedStyle(pathElement).strokeDashoffset;
      console.log(`Stroke ${index + 1} current dashoffset:`, currentDashOffset);
      
      // Make strokes visible immediately by setting dashoffset to 0
      pathElement.style.strokeDashoffset = '0';
      pathElement.style.animation = 'none'; // Disable animation
      
      // Also ensure stroke is visible
      pathElement.style.stroke = '#000';
      pathElement.style.strokeWidth = '80px';
      pathElement.style.fill = 'none';
      pathElement.style.opacity = '1';
      pathElement.style.strokeLinecap = 'round';
      pathElement.style.strokeLinejoin = 'round';
      
      // Log new state after modification
      const newDashOffset = window.getComputedStyle(pathElement).strokeDashoffset;
      console.log(`Stroke ${index + 1} new dashoffset:`, newDashOffset);
    });
    
    // Also handle static paths (background strokes)
    const staticPaths = svgContainer.querySelectorAll('path[id]');
    console.log('Found', staticPaths.length, 'static stroke paths');
    
    staticPaths.forEach((path, index) => {
      const pathElement = path as HTMLElement;
      
      // Make static strokes visible with light gray
      pathElement.style.stroke = '#ddd';
      pathElement.style.strokeWidth = '22px';
      pathElement.style.fill = 'none';
      pathElement.style.opacity = '0.4';
      pathElement.style.strokeLinecap = 'round';
      pathElement.style.strokeLinejoin = 'round';
      
      console.log(`Static stroke ${index + 1} styled`);
    });
    
    currentStroke = totalStrokes;
    isAnimating = false;
    
    console.log('Showed all AnimCJK strokes by setting stroke-dashoffset to 0');
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
        <div class="kanji-card" on:click={() => openKanjiModal(kanji)} on:keydown={(e) => e.key === 'Enter' && openKanjiModal(kanji)} tabindex="0" role="button">
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

<!-- Kanji Details Modal -->
{#if modalOpen && selectedKanji}
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div class="modal-overlay" on:click={closeModal} on:keydown={handleKeydown} tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="modal-title">
    <div class="modal-content" on:click|stopPropagation on:keydown|stopPropagation role="document">
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
                {#if kanjiSvg && !kanjiSvg.includes('class="acjk"')}
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
                    on:input={() => changeAnimationSpeed(animationSpeed)}
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
                <div class="modal-meta-item">
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
