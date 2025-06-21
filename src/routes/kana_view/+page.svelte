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

    // Define the traditional kana chart structure
    const kanaChart: Record<'hiragana' | 'katakana' | 'romaji', string[][]> = {
      hiragana: [
        ['あ', 'い', 'う', 'え', 'お'],
        ['か', 'き', 'く', 'け', 'こ'],
        ['が', 'ぎ', 'ぐ', 'げ', 'ご'],
        ['さ', 'し', 'す', 'せ', 'そ'],
        ['ざ', 'じ', 'ず', 'ぜ', 'ぞ'],
        ['た', 'ち', 'つ', 'て', 'と'],
        ['だ', 'ぢ', 'づ', 'で', 'ど'],
        ['な', 'に', 'ぬ', 'ね', 'の'],
        ['は', 'ひ', 'ふ', 'へ', 'ほ'],
        ['ば', 'び', 'ぶ', 'べ', 'ぼ'],
        ['ぱ', 'ぴ', 'ぷ', 'ぺ', 'ぽ'],
        ['ま', 'み', 'む', 'め', 'も'],
        ['や', '', 'ゆ', '', 'よ'],
        ['ら', 'り', 'る', 'れ', 'ろ'],
        ['わ', '', '', '', 'を'],
        ['ん', '', '', '', '']
      ],
      katakana: [
        ['ア', 'イ', 'ウ', 'エ', 'オ'],
        ['カ', 'キ', 'ク', 'ケ', 'コ'],
        ['ガ', 'ギ', 'グ', 'ゲ', 'ゴ'],
        ['サ', 'シ', 'ス', 'セ', 'ソ'],
        ['ザ', 'ジ', 'ズ', 'ゼ', 'ゾ'],
        ['タ', 'チ', 'ツ', 'テ', 'ト'],
        ['ダ', 'ヂ', 'ヅ', 'デ', 'ド'],
        ['ナ', 'ニ', 'ヌ', 'ネ', 'ノ'],
        ['ハ', 'ヒ', 'フ', 'ヘ', 'ホ'],
        ['バ', 'ビ', 'ブ', 'ベ', 'ボ'],
        ['パ', 'ピ', 'プ', 'ペ', 'ポ'],
        ['マ', 'ミ', 'ム', 'メ', 'モ'],
        ['ヤ', '', 'ユ', '', 'ヨ'],
        ['ラ', 'リ', 'ル', 'レ', 'ロ'],
        ['ワ', '', '', '', 'ヲ'],
        ['ン', '', '', '', '']
      ],
      romaji: [
        ['a', 'i', 'u', 'e', 'o'],
        ['ka', 'ki', 'ku', 'ke', 'ko'],
        ['ga', 'gi', 'gu', 'ge', 'go'],
        ['sa', 'shi', 'su', 'se', 'so'],
        ['za', 'ji', 'zu', 'ze', 'zo'],
        ['ta', 'chi', 'tsu', 'te', 'to'],
        ['da', 'ji', 'zu', 'de', 'do'],
        ['na', 'ni', 'nu', 'ne', 'no'],
        ['ha', 'hi', 'fu', 'he', 'ho'],
        ['ba', 'bi', 'bu', 'be', 'bo'],
        ['pa', 'pi', 'pu', 'pe', 'po'],
        ['ma', 'mi', 'mu', 'me', 'mo'],
        ['ya', '', 'yu', '', 'yo'],
        ['ra', 'ri', 'ru', 're', 'ro'],
        ['wa', '', '', '', 'wo'],
        ['n', '', '', '', '']
      ]
    };

    onMount(async () => {
      try {
        const [hira, kata] = await invoke<[Kana[], Kana[]]>("get_kana");
        hiraganaList = hira;
        katakanaList = kata;
      } catch (err) {
        console.error("Failed to load kana:", err);
      }
    });

    function showDetails(kana: string, romaji: string) {
      if (kana) {
        alert(`${kana} (${romaji}) - ${activeTab}`);
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
    
  <main class="kana-chart-container">
    <!-- Column headers -->
    <div class="chart-header">
      <div class="header-cell"></div>
      <div class="header-cell">a</div>
      <div class="header-cell">i</div>
      <div class="header-cell">u</div>
      <div class="header-cell">e</div>
      <div class="header-cell">o</div>
    </div>
    
    <!-- Kana chart grid -->
    <div class="kana-chart">
      {#each kanaChart[activeTab.toLowerCase() as 'hiragana' | 'katakana'] as row, rowIndex}
        <div class="kana-row">
          <!-- Row label (first consonant of each row) -->
          <div class="row-label">
            {#if rowIndex < kanaChart.romaji.length && kanaChart.romaji[rowIndex][0]}
              {kanaChart.romaji[rowIndex][0].charAt(0)}
            {/if}
          </div>
          
          <!-- Kana characters -->
          {#each row as kana, colIndex}
            <div class="kana-cell">
              {#if kana}
                <button 
                  class="kana-button" 
                  on:click={() => showDetails(kana, kanaChart.romaji[rowIndex][colIndex])}
                  type="button" 
                  aria-label={`${kana} (${kanaChart.romaji[rowIndex][colIndex]})`}
                >
                  <div class="kana-char">{kana}</div>
                  <div class="romaji">{kanaChart.romaji[rowIndex][colIndex]}</div>
                </button>
              {/if}
            </div>
          {/each}
        </div>
      {/each}
    </div>
  </main>