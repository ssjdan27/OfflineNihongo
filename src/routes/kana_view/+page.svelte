<script lang="ts">
    import { onDestroy, tick } from 'svelte';
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import './kana-view.css';

    let activeTab: "Hiragana" | "Katakana" = "Hiragana";
    let activeSection: "gojuon" | "dakuon" | "yoon" = "gojuon";

    // Modal state
    let showModal = false;
    let selectedKana = '';
    let selectedRomaji = '';
    let selectedScript = '';
    let modalRequest = 0; // increments on every open/close so stale loads are ignored

    // Stroke order diagram: one SVG per character (yōon like きゃ have two)
    let kanaSvgs: string[] = [];
    let svgLoading = false;

    // Animation state
    let animationSpeed = 1000; // milliseconds per stroke
    let isAnimating = false;
    let currentStrokeIndex = 0;
    let totalStrokes = 0;
    let animationRun = 0; // increments to cancel a running animation
    let animationTimer: ReturnType<typeof setTimeout> | null = null;

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

    onDestroy(stopAnimation);

    async function showDetails(kana: string, romaji: string) {
      if (!kana) return;
      const request = ++modalRequest;
      stopAnimation();
      selectedKana = kana;
      selectedRomaji = romaji;
      selectedScript = activeTab;
      showModal = true;
      svgLoading = true;
      kanaSvgs = [];
      totalStrokes = 0;
      currentStrokeIndex = 0;

      // The diagrams are per character, so load one for each kana in a yōon
      let svgs: string[] = [];
      try {
        svgs = await Promise.all(
          Array.from(kana).map((character) =>
            invoke<string>('get_kana_svg', { character })
          )
        );
      } catch (error) {
        console.error('Error loading kana SVG:', error);
        svgs = [];
      }
      if (request !== modalRequest) return;

      kanaSvgs = svgs;
      svgLoading = false;
      if (svgs.length === 0) return;

      await tick();
      if (request !== modalRequest) return;
      const wrapper = document.querySelector('.kana-svg-wrapper');
      if (!wrapper) return;
      wrapper.innerHTML = svgs.join('');
      prepareDiagram(wrapper);
      // Draw the kana when the modal opens, like the AnimCJK default
      startAnimation();
    }

    function getDiagrams(): SVGSVGElement[] {
      return Array.from(document.querySelectorAll<SVGSVGElement>('.kana-svg-wrapper svg'));
    }

    // AnimCJK splits some strokes into several clipped pieces that share the
    // same stroke number (the --d delay), e.g. あ has 3 strokes but 4 paths.
    // Group them so each stroke is drawn, and counted, once.
    function getStrokeGroups(): SVGPathElement[][] {
      const groups: SVGPathElement[][] = [];
      for (const svg of getDiagrams()) {
        let lastKey: string | null = null;
        for (const path of svg.querySelectorAll<SVGPathElement>('path[clip-path]')) {
          const key = /--d:\s*([\d.]+)s/.exec(path.getAttribute('style') ?? '')?.[1] ?? null;
          if (key !== null && key === lastKey) {
            groups[groups.length - 1].push(path);
          } else {
            groups.push([path]);
          }
          lastKey = key;
        }
      }
      return groups;
    }

    function prepareDiagram(wrapper: Element) {
      const diagrams = getDiagrams();
      const size = diagrams.length > 1 ? '130px' : '260px';
      for (const svg of diagrams) {
        svg.style.width = size;
        svg.style.height = size;
        svg.style.display = 'block';
        svg.style.backgroundColor = 'white';
      }
      wrapper.classList.toggle('multiple', diagrams.length > 1);

      // Take over from the SVG's own autoplay animation so the controls work
      for (const group of getStrokeGroups()) {
        for (const path of group) {
          path.style.animation = 'none';
          path.style.strokeDasharray = '3337';
        }
      }
      totalStrokes = getStrokeGroups().length;
    }

    function setStrokeVisible(group: SVGPathElement[], visible: boolean, animate: boolean) {
      // Draw over most of the step so there's a short pause between strokes
      const duration = (animationSpeed * 0.8) / 1000;
      for (const path of group) {
        path.style.transition = animate ? `stroke-dashoffset ${duration}s linear` : 'none';
        path.style.strokeDashoffset = visible ? '0' : '3339';
      }
    }

    function startAnimation() {
      const groups = getStrokeGroups();
      if (groups.length === 0) return;

      stopAnimation();
      const run = animationRun;
      groups.forEach((group) => setStrokeVisible(group, false, false));
      currentStrokeIndex = 0;
      isAnimating = true;

      // Each step reads animationSpeed, so the slider changes the pace of a
      // running animation.
      const step = () => {
        if (run !== animationRun) return;
        if (currentStrokeIndex >= groups.length) {
          animationTimer = null;
          isAnimating = false;
          return;
        }
        setStrokeVisible(groups[currentStrokeIndex], true, true);
        currentStrokeIndex += 1;
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
      stopAnimation();
      getStrokeGroups().forEach((group) => setStrokeVisible(group, false, false));
      currentStrokeIndex = 0;
    }

    function showAllStrokes() {
      stopAnimation();
      const groups = getStrokeGroups();
      groups.forEach((group) => setStrokeVisible(group, true, false));
      currentStrokeIndex = groups.length;
    }

    function closeModal() {
      modalRequest++;
      stopAnimation();
      showModal = false;
      svgLoading = false;
      kanaSvgs = [];
      selectedKana = '';
      selectedRomaji = '';
      selectedScript = '';
    }

    function handleKeydown(event: KeyboardEvent) {
      if (event.key === 'Escape' && showModal) {
        closeModal();
      }
    }

    // Close only when the backdrop itself is clicked, not the dialog
    function handleOverlayClick(event: MouseEvent) {
      if (event.target === event.currentTarget) {
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
    <!-- Escape is handled by the window listener below -->
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="modal-overlay" on:click={handleOverlayClick}>
      <div class="modal-content" role="dialog" aria-modal="true" aria-labelledby="kana-modal-title">
        <div class="modal-header">
          <h2 id="kana-modal-title">{selectedKana} ({selectedRomaji})</h2>
          <button class="modal-close" on:click={closeModal} aria-label="Close modal">×</button>
        </div>

        <div class="modal-body">
          <div class="modal-left">
            <div class="modal-kana-character">{selectedKana}</div>

            <div class="modal-svg-container">
              {#if svgLoading}
                <div class="svg-loading">Loading stroke order...</div>
              {:else if kanaSvgs.length > 0}
                <!-- The SVGs are injected by showDetails() -->
                <div class="kana-svg-wrapper"></div>
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
                      Stroke {currentStrokeIndex} of {totalStrokes}
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