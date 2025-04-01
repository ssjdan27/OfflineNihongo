<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { invoke } from '@tauri-apps/api/core';

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

onMount(async () => {
  try {
    const [hira, kata] = await invoke<[Kana[], Kana[]]>("get_kana");
    hiraganaList = hira;
    katakanaList = kata;
  } catch (err) {
    console.error("Failed to load kana:", err);
  }
});

  
    function showDetails(k: Kana) {
      alert(`${k.kana} (${k.romaji}) - ${k.script}`);
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
    
  <main class="kana-grid">
    {#each activeTab === "Hiragana" ? hiraganaList : katakanaList as k}
      <button class="kana-card" on:click={() => showDetails(k)} type="button" aria-label={`Details for ${k.kana}`}>
        <div class="kana-char">{k.kana}</div>
        <div class="romaji">{k.romaji}</div>
      </button>
    {/each}
  </main>
    
  <style>
    .nav-bar {
      display: flex;
      justify-content: flex-start;
      padding: 1rem;
    }
    
    .home-button {
      padding: 0.5rem 1rem;
      font-size: 1rem;
      background-color: #f3f3f3;
      border: 1px solid #ccc;
      border-radius: 6px;
      cursor: pointer;
    }
    
    .home-button:hover {
      background-color: #e0e0e0;
    }
  
    .tab-container {
      display: flex;
      justify-content: center;
      gap: 1rem;
      margin: 2rem 0 1rem;
    }
    
    button {
      padding: 0.5rem 1rem;
      font-size: 1rem;
      border: 2px solid #ccc;
      background: white;
      cursor: pointer;
      border-radius: 6px;
      transition: background 0.2s;
    }
    
    button:hover {
      background-color: #f0f0f0;
    }
    
    .active-tab {
      background-color: #396cd8;
      color: white;
      border-color: #396cd8;
    }
    
    .kana-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
      gap: 1rem;
      padding: 2rem;
    }
    
    .kana-card {
      text-align: center;
      padding: 1rem;
      border: 1px solid #ccc;
      border-radius: 8px;
      cursor: pointer;
      background: #fff;
      box-shadow: 0 1px 5px rgba(0,0,0,0.1);
      transition: transform 0.2s ease;
    }
    
    .kana-card:hover {
      transform: scale(1.05);
    }
    
    .kana-char {
      font-size: 2rem;
    }
    
    .romaji {
      font-size: 0.9rem;
      color: #555;
    }
  </style>