<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import './kana-view.css';

    type Kana = {
      kana: string;
      romaji: string;
      script: "Hiragana" | "Katakana";
      row: string;
      col: string;
    };

    let hiraganaList: Kana[] = [];
    let katakanaList: Kana[] = [];
    let activeTab: "Hiragana" | "Katakana" = "Hiragana";
    let activeSection: "gojuon" | "dakuon" | "yoon" = "gojuon";

    // Modal state
    let showModal = false;
    let selectedKana = '';
    let selectedRomaji = '';
    let selectedScript = '';

    // Animation state
    let animationId: number | null = null;
    let animationSpeed = 1000; // Default speed in milliseconds
    let isAnimating = false;
    let currentStrokeIndex = 0;
    let currentStrokes: SVGPathElement[] = [];

    // Define the traditional kana chart structure organized by sections
    const kanaChart = {
      hiragana: {
        gojuon: [
          ['あ', 'い', 'う', 'え', 'お'],
          ['か', 'き', 'く', 'け', 'こ'],
          ['さ', 'し', 'す', 'せ', 'そ'],
          ['た', 'ち', 'つ', 'て', 'と'],
          ['な', 'に', 'ぬ', 'ね', 'の'],
          ['は', 'ひ', 'ふ', 'へ', 'ほ'],
          ['ま', 'み', 'む', 'め', 'も'],
          ['や', '', 'ゆ', '', 'よ'],
          ['ら', 'り', 'る', 'れ', 'ろ'],
          ['わ', '', '', '', 'を'],
          ['', '', 'ん', '', '']
        ],
        dakuon: [
          ['が', 'ぎ', 'ぐ', 'げ', 'ご'],
          ['ざ', 'じ', 'ず', 'ぜ', 'ぞ'],
          ['だ', 'ぢ', 'づ', 'で', 'ど'],
          ['ば', 'び', 'ぶ', 'べ', 'ぼ'],
          ['ぱ', 'ぴ', 'ぷ', 'ぺ', 'ぽ']
        ],
        yoon: [
          ['きゃ', 'きゅ', 'きょ'],
          ['ぎゃ', 'ぎゅ', 'ぎょ'],
          ['しゃ', 'しゅ', 'しょ'],
          ['じゃ', 'じゅ', 'じょ'],
          ['ちゃ', 'ちゅ', 'ちょ'],
          ['ぢゃ', 'ぢゅ', 'ぢょ'],
          ['にゃ', 'にゅ', 'にょ'],
          ['ひゃ', 'ひゅ', 'ひょ'],
          ['びゃ', 'びゅ', 'びょ'],
          ['ぴゃ', 'ぴゅ', 'ぴょ'],
          ['みゃ', 'みゅ', 'みょ'],
          ['りゃ', 'りゅ', 'りょ']
        ]
      },
      katakana: {
        gojuon: [
          ['ア', 'イ', 'ウ', 'エ', 'オ'],
          ['カ', 'キ', 'ク', 'ケ', 'コ'],
          ['サ', 'シ', 'ス', 'セ', 'ソ'],
          ['タ', 'チ', 'ツ', 'テ', 'ト'],
          ['ナ', 'ニ', 'ヌ', 'ネ', 'ノ'],
          ['ハ', 'ヒ', 'フ', 'ヘ', 'ホ'],
          ['マ', 'ミ', 'ム', 'メ', 'モ'],
          ['ヤ', '', 'ユ', '', 'ヨ'],
          ['ラ', 'リ', 'ル', 'レ', 'ロ'],
          ['ワ', '', '', '', 'ヲ'],
          ['', '', 'ン', '', '']
        ],
        dakuon: [
          ['ガ', 'ギ', 'グ', 'ゲ', 'ゴ'],
          ['ザ', 'ジ', 'ズ', 'ゼ', 'ゾ'],
          ['ダ', 'ヂ', 'ヅ', 'デ', 'ド'],
          ['バ', 'ビ', 'ブ', 'ベ', 'ボ'],
          ['パ', 'ピ', 'プ', 'ペ', 'ポ']
        ],
        yoon: [
          ['キャ', 'キュ', 'キョ'],
          ['ギャ', 'ギュ', 'ギョ'],
          ['シャ', 'シュ', 'ショ'],
          ['ジャ', 'ジュ', 'ジョ'],
          ['チャ', 'チュ', 'チョ'],
          ['ヂャ', 'ヂュ', 'ヂョ'],
          ['ニャ', 'ニュ', 'ニョ'],
          ['ヒャ', 'ヒュ', 'ヒョ'],
          ['ビャ', 'ビュ', 'ビョ'],
          ['ピャ', 'ピュ', 'ピョ'],
          ['ミャ', 'ミュ', 'ミョ'],
          ['リャ', 'リュ', 'リョ']
        ]
      },
      romaji: {
        gojuon: [
          ['a', 'i', 'u', 'e', 'o'],
          ['ka', 'ki', 'ku', 'ke', 'ko'],
          ['sa', 'shi', 'su', 'se', 'so'],
          ['ta', 'chi', 'tsu', 'te', 'to'],
          ['na', 'ni', 'nu', 'ne', 'no'],
          ['ha', 'hi', 'fu', 'he', 'ho'],
          ['ma', 'mi', 'mu', 'me', 'mo'],
          ['ya', '', 'yu', '', 'yo'],
          ['ra', 'ri', 'ru', 're', 'ro'],
          ['wa', '', '', '', 'wo'],
          ['', '', 'n', '', '']
        ],
        dakuon: [
          ['ga', 'gi', 'gu', 'ge', 'go'],
          ['za', 'ji', 'zu', 'ze', 'zo'],
          ['da', 'ji', 'zu', 'de', 'do'],
          ['ba', 'bi', 'bu', 'be', 'bo'],
          ['pa', 'pi', 'pu', 'pe', 'po']
        ],
        yoon: [
          ['kya', 'kyu', 'kyo'],
          ['gya', 'gyu', 'gyo'],
          ['sha', 'shu', 'sho'],
          ['ja', 'ju', 'jo'],
          ['cha', 'chu', 'cho'],
          ['ja', 'ju', 'jo'],
          ['nya', 'nyu', 'nyo'],
          ['hya', 'hyu', 'hyo'],
          ['bya', 'byu', 'byo'],
          ['pya', 'pyu', 'pyo'],
          ['mya', 'myu', 'myo'],
          ['rya', 'ryu', 'ryo']
        ]
      }
    };

    // Get section titles
    const getSectionTitle = () => {
      switch(activeSection) {
        case 'gojuon': return 'Gojuon (basic kana)';
        case 'dakuon': return 'Dakuon & Handakuon';
        case 'yoon': return 'Yoon';
        default: return '';
      }
    };

    // Get row labels based on section
    const getRowLabel = (rowIndex: number) => {
      const script = activeTab.toLowerCase() as 'hiragana' | 'katakana';
      const romaji = kanaChart.romaji[activeSection];
      
      if (activeSection === 'gojuon') {
        const labels = ['', 'k~', 's~', 't~', 'n~', 'h~', 'm~', 'y~', 'r~', 'w~', ''];
        return labels[rowIndex] || '';
      } else if (activeSection === 'dakuon') {
        const labels = ['g~', 'z~', 'd~', 'b~', 'p~'];
        return labels[rowIndex] || '';
      } else if (activeSection === 'yoon') {
        const labels = ['ky~', 'gy~', 'sh~', 'j~', 'ch~', 'j~', 'ny~', 'hy~', 'by~', 'py~', 'my~', 'ry~'];
        return labels[rowIndex] || '';
      }
      return '';
    };

    onMount(async () => {
      try {
        // using static data for kana so we don't need to fetch
      } catch (err) {
        console.error("Failed to load kana:", err);
      }
    });

    function showDetails(kana: string, romaji: string) {
      if (kana) {
        selectedKana = kana;
        selectedRomaji = romaji;
        selectedScript = activeTab;
        showModal = true;
        loadKanaSvg();
      }
    }

    // SVG loading and injection
    let kanaSvg = '';
    let svgLoading = false;
    
    async function loadKanaSvg() {
      try {
        console.log(`Loading SVG for ${selectedKana}`);
        svgLoading = true;
        
        const svgContent = await invoke('get_kana_svg', { 
          character: selectedKana 
        }) as string;
        
        console.log('SVG content received:', svgContent ? 'Yes' : 'No');
        if (svgContent) {
          console.log('SVG content preview:', svgContent.substring(0, 200) + '...');
        }
        
        kanaSvg = svgContent || '';
        
        // Wait for DOM to update, then inject and initialize
        setTimeout(() => {
          injectAndInitializeAnimation();
        }, 50);
        
      } catch (error) {
        console.error('Error loading kana SVG:', error);
        kanaSvg = '';
      } finally {
        svgLoading = false;
      }
    }

    // SVG injection and animation initialization (matching kanji modal)
    function injectAndInitializeAnimation() {
      if (!kanaSvg) {
        console.log('No kanaSvg content available');
        return;
      }
      
      console.log('InjectAndInitializeAnimation called, kanaSvg length:', kanaSvg.length);
      
      // Wait for DOM to be ready
      const svgWrapper = document.querySelector('.svg-wrapper');
      
      if (!svgWrapper) {
        console.warn('SVG wrapper not found');
        return;
      }
      
      // Check if this is an AnimCJK SVG by looking for the acjk class in the SVG content
      const isAnimCJK = kanaSvg.includes('class="acjk"');
      
      console.log('Is AnimCJK SVG:', isAnimCJK);
      
      if (isAnimCJK) {
        // AnimCJK SVG - use fetch/innerHTML method as recommended
        console.log('AnimCJK SVG detected - using direct innerHTML injection');
        
        // Clear the wrapper and inject the SVG directly
        svgWrapper.innerHTML = kanaSvg;
        
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
          
          // Filter out duplicate strokes based on path data
          const uniqueStrokes = [];
          const seenPaths = new Set();
          
          for (const stroke of strokePaths) {
            const pathData = stroke.getAttribute('d');
            if (pathData && !seenPaths.has(pathData)) {
              seenPaths.add(pathData);
              uniqueStrokes.push(stroke as SVGPathElement);
            }
          }
          
          currentStrokes = uniqueStrokes;
          console.log(`Found ${strokePaths.length} total animated strokes, ${staticPaths.length} static paths, ${currentStrokes.length} unique strokes for AnimCJK kana`);
          
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
          setTimeout(() => {
            showAllAnimCJKStrokes();
          }, 100);
        } else {
          console.error('Failed to find injected SVG element');
        }
      } else {
        console.log('Non-AnimCJK SVG detected - using fallback method');
        
        // Clear the wrapper (let Svelte handle it via {@html})
        svgWrapper.innerHTML = '';
        
        // Initialize fallback animation after DOM updates
        setTimeout(() => {
          initializeFallbackAnimation();
        }, 50);
      }
    }

    // Fallback animation for non-AnimCJK SVGs
    function initializeFallbackAnimation() {
      const svgContainer = document.querySelector('.svg-wrapper svg');
      
      console.log('SVG container element:', svgContainer);
      
      if (!svgContainer) {
        console.warn('SVG container not found for fallback');
        return;
      }
      
      console.log('Fallback SVG detected - using manual animation');
      
      // Find all path elements
      let strokePaths = svgContainer.querySelectorAll('path');
      
      // Filter out duplicate strokes based on path data
      const uniqueStrokes = [];
      const seenPaths = new Set();
      
      for (const stroke of strokePaths) {
        const pathData = stroke.getAttribute('d');
        if (pathData && !seenPaths.has(pathData)) {
          seenPaths.add(pathData);
          uniqueStrokes.push(stroke as SVGPathElement);
        }
      }
      
      currentStrokes = uniqueStrokes;
      console.log(`Found ${strokePaths.length} total paths, ${currentStrokes.length} unique strokes for fallback kana`);
      
      // Reset animation state
      resetAnimation();
    }

    // Show all strokes for AnimCJK SVGs
    function showAllAnimCJKStrokes() {
      const svgContainer = document.querySelector('.svg-wrapper svg');
      if (!svgContainer) return;
      
      const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
      const staticPaths = svgContainer.querySelectorAll('path[id]');
      
      console.log('Showing all AnimCJK strokes');
      
      // Show all animated strokes
      strokePaths.forEach((path, index) => {
        const pathElement = path as HTMLElement;
        pathElement.style.opacity = '1';
        console.log(`Showed animated stroke ${index + 1}`);
      });
      
      // Show all static strokes
      staticPaths.forEach((path, index) => {
        const pathElement = path as HTMLElement;
        pathElement.style.opacity = '1';
        console.log(`Showed static stroke ${index + 1}`);
      });
    }

    function closeModal() {
      showModal = false;
      stopAnimation();
      selectedKana = '';
      selectedRomaji = '';
      selectedScript = '';
    }

    // Animation control functions (matching kanji modal)
    function startAnimation() {
      const svgContainer = document.querySelector('.svg-wrapper svg');
      if (!svgContainer) return;
      
      const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
      
      if (strokePaths.length === 0) {
        console.log('No animated strokes found');
        return;
      }
      
      console.log('Starting AnimCJK animation');
      
      isAnimating = true;
      currentStrokeIndex = 0;
      
      // Hide all strokes first
      strokePaths.forEach(path => {
        (path as HTMLElement).style.opacity = '0';
      });
      
      // Start animation
      animateNextStroke();
    }

    function playAnimation() {
      startAnimation();
    }

    function animateNextStroke() {
      const svgContainer = document.querySelector('.svg-wrapper svg');
      if (!svgContainer) return;
      
      const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
      
      console.log(`animateNextStroke called, currentStrokeIndex: ${currentStrokeIndex}, total strokes: ${strokePaths.length}`);
      
      if (currentStrokeIndex >= strokePaths.length) {
        console.log('Animation finished');
        isAnimating = false;
        return;
      }
      
      const stroke = strokePaths[currentStrokeIndex] as HTMLElement;
      console.log(`Showing stroke ${currentStrokeIndex + 1}:`, stroke);
      stroke.style.opacity = '1';
      
      currentStrokeIndex++;
      
      if (currentStrokeIndex < strokePaths.length) {
        console.log(`Setting timeout for next stroke (${animationSpeed}ms)`);
        animationId = window.setTimeout(animateNextStroke, animationSpeed);
      } else {
        console.log('All strokes shown, animation complete');
        isAnimating = false;
      }
    }

    function stopAnimation() {
      if (animationId) {
        clearTimeout(animationId);
        animationId = null;
      }
      isAnimating = false;
    }

    function resetAnimation() {
      const svgContainer = document.querySelector('.svg-wrapper svg');
      if (!svgContainer) return;
      
      stopAnimation();
      currentStrokeIndex = 0;
      
      // Hide all animated strokes
      const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
      strokePaths.forEach(path => {
        (path as HTMLElement).style.opacity = '0';
      });
    }

    function showAllStrokes() {
      const svgContainer = document.querySelector('.svg-wrapper svg');
      if (!svgContainer) return;
      
      console.log('Show all strokes clicked');
      
      // Show all animated strokes
      const strokePaths = svgContainer.querySelectorAll('path[clip-path]');
      strokePaths.forEach((path, index) => {
        console.log(`Setting stroke ${index + 1} opacity to 1`);
        (path as HTMLElement).style.opacity = '1';
      });
      
      // Show all static strokes
      const staticPaths = svgContainer.querySelectorAll('path[id]');
      staticPaths.forEach((path, index) => {
        console.log(`Setting static stroke ${index + 1} opacity to 1`);
        (path as HTMLElement).style.opacity = '1';
      });
      
      stopAnimation();
    }

    function toggleShowAllStrokes() {
      showAllStrokes();
    }

    function changeAnimationSpeed(speed: number) {
      animationSpeed = speed;
      console.log(`Animation speed changed to ${speed}ms`);
    }

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        closeModal();
      }
    }
    
    function goToHome() {
      goto('/');
    }
  </script>
  
  <div class="nav-bar">
    <button class="home-button" on:click={goToHome}>Back to Home</button>
  </div>
    
  <div class="tab-container">
    <button
      class:active-tab={activeTab === "Hiragana"}
      on:click={() => (activeTab = "Hiragana")}
    >
      Hiragana
    </button>
    <button
      class:active-tab={activeTab === "Katakana"}
      on:click={() => (activeTab = "Katakana")}
    >
      Katakana
    </button>
  </div>

  <!-- Section tabs -->
  <div class="section-container">
    <button
      class:active-section={activeSection === "gojuon"}
      on:click={() => (activeSection = "gojuon")}
    >
      Gojuon
    </button>
    <button
      class:active-section={activeSection === "dakuon"}
      on:click={() => (activeSection = "dakuon")}
    >
      Dakuon & Handakuon
    </button>
    <button
      class:active-section={activeSection === "yoon"}
      on:click={() => (activeSection = "yoon")}
    >
      Yoon
    </button>
  </div>

  <!-- Section title -->
  <div class="section-title">
    <h2>{getSectionTitle()}</h2>
  </div>
    
  <main class="kana-chart-container">
    <!-- Column headers for gojuon and dakuon -->
    {#if activeSection === 'gojuon' || activeSection === 'dakuon'}
      <div class="chart-header">
        <div class="header-cell"></div>
        <div class="header-cell">~a</div>
        <div class="header-cell">~i</div>
        <div class="header-cell">~u</div>
        <div class="header-cell">~e</div>
        <div class="header-cell">~o</div>
      </div>
    {:else if activeSection === 'yoon'}
      <!-- Column headers for yoon -->
      <div class="chart-header">
        <div class="header-cell"></div>
        <div class="header-cell">~ya</div>
        <div class="header-cell">~yu</div>
        <div class="header-cell">~yo</div>
      </div>
    {/if}
    
    <!-- Kana chart grid -->
    <div class="kana-chart">
      {#each kanaChart[activeTab.toLowerCase() as 'hiragana' | 'katakana'][activeSection] as row, rowIndex}
        <div class="kana-row">
          <!-- Row label -->
          <div class="row-label">
            {getRowLabel(rowIndex)}
          </div>
          
          <!-- Kana characters -->
          {#each row as kana, colIndex}
            <div class="kana-cell">
              {#if kana}
                <button 
                  class="kana-button" 
                  on:click={() => showDetails(kana, kanaChart.romaji[activeSection][rowIndex][colIndex])}
                  type="button" 
                  aria-label={`${kana} (${kanaChart.romaji[activeSection][rowIndex][colIndex]})`}
                >
                  <div class="kana-char">{kana}</div>
                  <div class="romaji">{kanaChart.romaji[activeSection][rowIndex][colIndex]}</div>
                </button>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    </div>
  </main>

  <!-- Modal -->
  {#if showModal}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="modal-overlay" on:click={closeModal}>
      <div class="modal-content" on:click|stopPropagation>
        <div class="modal-header">
          <h2>{selectedKana} ({selectedRomaji})</h2>
          <button class="modal-close" on:click={closeModal} aria-label="Close modal">×</button>
        </div>
        
        <div class="modal-body">
          <div class="modal-left">
            <div class="modal-kana-character">{selectedKana}</div>
            
            <div class="modal-svg-container">
              {#if svgLoading}
                <div class="svg-loading">Loading stroke order...</div>
              {:else if kanaSvg}
                <div class="svg-wrapper">
                  <!-- AnimCJK SVGs are injected via JavaScript, fallback SVGs use {@html} -->
                  {#if kanaSvg && !kanaSvg.includes('class="acjk"')}
                    {@html kanaSvg}
                  {/if}
                </div>
                <!-- Animation Controls -->
                <div class="animation-controls">
                  <div class="animation-buttons">
                    <button class="control-btn" on:click={startAnimation} disabled={isAnimating}>
                      {isAnimating ? 'Playing...' : 'Play'}
                    </button>
                    <button class="control-btn" on:click={stopAnimation} disabled={!isAnimating}>
                      Stop
                    </button>
                    <button class="control-btn" on:click={resetAnimation}>
                      Reset
                    </button>
                    <button class="control-btn" on:click={showAllStrokes}>
                      Show All
                    </button>
                  </div>
                  
                  <div class="animation-info">
                    <span class="stroke-counter">
                      Stroke {currentStrokeIndex} of {currentStrokes.length}
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
              <h3>Character Information</h3>
              <div class="modal-reading">
                <span class="modal-label">Romaji:</span> {selectedRomaji}
              </div>
              <div class="modal-reading">
                <span class="modal-label">Script:</span> {selectedScript}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}

<svelte:window on:keydown={handleKeydown} />