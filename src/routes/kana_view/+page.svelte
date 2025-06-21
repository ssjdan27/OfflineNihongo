<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
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