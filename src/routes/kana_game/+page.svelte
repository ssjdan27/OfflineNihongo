<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import './kana-game.css';

  interface KanaChar {
    character: string;
    romaji: string;
    kana_type: string;
    sound_type: string;
    complexity: string;
  }

  let kanaData: KanaChar[] = [];
  let loading = true;
  let error = '';
  
  // Game settings
  let gameMode: 'hiragana' | 'katakana' | 'mixed' = 'hiragana';
  let soundTypes: string[] = ['seion']; // Can include: seion, dakuon, handakuon
  let complexity: 'basic' | 'combination' | 'mixed' = 'basic';
  let gameStarted = false;
  let gameCompleted = false;
  
  // Game state
  let currentKana: KanaChar | null = null;
  let userInput = '';
  let currentIndex = 0;
  let gameKana: KanaChar[] = [];
  let correctCount = 0;
  let totalCount = 0;
  
  // Timing
  let startTime: number = 0;
  let endTime: number = 0;
  let elapsedTime: number = 0;
  let timerInterval: ReturnType<typeof setInterval>;
  
  // Results
  let bestTimes = {
    'hiragana-seion-basic': localStorage.getItem('best-hiragana-seion-basic-time') ? parseInt(localStorage.getItem('best-hiragana-seion-basic-time')!) : null,
    'hiragana-seion-combination': localStorage.getItem('best-hiragana-seion-combination-time') ? parseInt(localStorage.getItem('best-hiragana-seion-combination-time')!) : null,
    'hiragana-dakuon-basic': localStorage.getItem('best-hiragana-dakuon-basic-time') ? parseInt(localStorage.getItem('best-hiragana-dakuon-basic-time')!) : null,
    'hiragana-dakuon-combination': localStorage.getItem('best-hiragana-dakuon-combination-time') ? parseInt(localStorage.getItem('best-hiragana-dakuon-combination-time')!) : null,
    'hiragana-handakuon-basic': localStorage.getItem('best-hiragana-handakuon-basic-time') ? parseInt(localStorage.getItem('best-hiragana-handakuon-basic-time')!) : null,
    'hiragana-handakuon-combination': localStorage.getItem('best-hiragana-handakuon-combination-time') ? parseInt(localStorage.getItem('best-hiragana-handakuon-combination-time')!) : null,
    'katakana-seion-basic': localStorage.getItem('best-katakana-seion-basic-time') ? parseInt(localStorage.getItem('best-katakana-seion-basic-time')!) : null,
    'katakana-seion-combination': localStorage.getItem('best-katakana-seion-combination-time') ? parseInt(localStorage.getItem('best-katakana-seion-combination-time')!) : null,
    'katakana-dakuon-basic': localStorage.getItem('best-katakana-dakuon-basic-time') ? parseInt(localStorage.getItem('best-katakana-dakuon-basic-time')!) : null,
    'katakana-dakuon-combination': localStorage.getItem('best-katakana-dakuon-combination-time') ? parseInt(localStorage.getItem('best-katakana-dakuon-combination-time')!) : null,
    'katakana-handakuon-basic': localStorage.getItem('best-katakana-handakuon-basic-time') ? parseInt(localStorage.getItem('best-katakana-handakuon-basic-time')!) : null,
    'katakana-handakuon-combination': localStorage.getItem('best-katakana-handakuon-combination-time') ? parseInt(localStorage.getItem('best-katakana-handakuon-combination-time')!) : null,
    // Mixed modes
    'mixed-seion-basic': localStorage.getItem('best-mixed-seion-basic-time') ? parseInt(localStorage.getItem('best-mixed-seion-basic-time')!) : null,
    'mixed-seion-combination': localStorage.getItem('best-mixed-seion-combination-time') ? parseInt(localStorage.getItem('best-mixed-seion-combination-time')!) : null,
    'mixed-dakuon-basic': localStorage.getItem('best-mixed-dakuon-basic-time') ? parseInt(localStorage.getItem('best-mixed-dakuon-basic-time')!) : null,
    'mixed-dakuon-combination': localStorage.getItem('best-mixed-dakuon-combination-time') ? parseInt(localStorage.getItem('best-mixed-dakuon-combination-time')!) : null,
    'mixed-handakuon-basic': localStorage.getItem('best-mixed-handakuon-basic-time') ? parseInt(localStorage.getItem('best-mixed-handakuon-basic-time')!) : null,
    'mixed-handakuon-combination': localStorage.getItem('best-mixed-handakuon-combination-time') ? parseInt(localStorage.getItem('best-mixed-handakuon-combination-time')!) : null,
  };

  onMount(async () => {
    await loadKanaData();
  });

  async function loadKanaData() {
    try {
      loading = true;
      error = '';
      kanaData = await invoke('get_kana_data');
      console.log('Loaded kana data:', kanaData.length);
    } catch (err) {
      error = `Failed to load kana data: ${err}`;
      console.error('Error loading kana data:', err);
    } finally {
      loading = false;
    }
  }

  function startGame() {
    // Filter kana based on game mode, sound types, and complexity
    let filteredKana = [...kanaData];
    
    // Filter by kana type (hiragana/katakana/mixed)
    if (gameMode !== 'mixed') {
      filteredKana = filteredKana.filter(k => k.kana_type === gameMode);
    }
    
    // Filter by sound types
    filteredKana = filteredKana.filter(k => soundTypes.includes(k.sound_type));
    
    // Filter by complexity
    if (complexity !== 'mixed') {
      filteredKana = filteredKana.filter(k => k.complexity === complexity);
    }
    
    // Shuffle the kana
    gameKana = shuffleArray(filteredKana);
    
    // Reset game state
    currentIndex = 0;
    correctCount = 0;
    totalCount = gameKana.length;
    userInput = '';
    gameStarted = true;
    gameCompleted = false;
    
    // Start timer
    startTime = Date.now();
    elapsedTime = 0;
    timerInterval = setInterval(updateTimer, 10);
    
    // Set first kana
    currentKana = gameKana[0];
  }

  function shuffleArray<T>(array: T[]): T[] {
    const shuffled = [...array];
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    return shuffled;
  }

  function updateTimer() {
    elapsedTime = Date.now() - startTime;
  }

  function handleInput(event: KeyboardEvent) {
    if (!gameStarted || gameCompleted) return;
    
    if (event.key === 'Enter') {
      checkAnswer();
    }
  }

  function checkAnswer() {
    if (!currentKana) return;
    
    const isCorrect = userInput.toLowerCase().trim() === currentKana.romaji.toLowerCase();
    
    if (isCorrect) {
      correctCount++;
      nextKana();
    } else {
      // Show feedback for incorrect answer but don't advance
      userInput = '';
    }
  }

  function nextKana() {
    currentIndex++;
    userInput = '';
    
    if (currentIndex >= gameKana.length) {
      // Game completed
      endGame();
    } else {
      currentKana = gameKana[currentIndex];
    }
  }

  function endGame() {
    gameCompleted = true;
    endTime = Date.now();
    clearInterval(timerInterval);
    
    const finalTime = endTime - startTime;
    
    // Generate key for best time based on current settings
    const gameKey = generateGameKey();
    
    // Check if this is a new best time
    const currentBest = bestTimes[gameKey];
    if (!currentBest || finalTime < currentBest) {
      bestTimes[gameKey] = finalTime;
      localStorage.setItem(`best-${gameKey}-time`, finalTime.toString());
    }
  }

  function generateGameKey(): string {
    const soundTypeKey = soundTypes.length === 1 ? soundTypes[0] : 'mixed';
    return `${gameMode}-${soundTypeKey}-${complexity}`;
  }

  function toggleSoundType(type: string) {
    if (soundTypes.includes(type)) {
      soundTypes = soundTypes.filter(t => t !== type);
    } else {
      soundTypes = [...soundTypes, type];
    }
    // Ensure at least one sound type is selected
    if (soundTypes.length === 0) {
      soundTypes = ['seion'];
    }
  }

  function getFilteredKanaCount(): number {
    let filtered = [...kanaData];
    
    if (gameMode !== 'mixed') {
      filtered = filtered.filter(k => k.kana_type === gameMode);
    }
    
    filtered = filtered.filter(k => soundTypes.includes(k.sound_type));
    
    if (complexity !== 'mixed') {
      filtered = filtered.filter(k => k.complexity === complexity);
    }
    
    return filtered.length;
  }

  function resetGame() {
    gameStarted = false;
    gameCompleted = false;
    currentKana = null;
    userInput = '';
    currentIndex = 0;
    correctCount = 0;
    totalCount = 0;
    elapsedTime = 0;
    clearInterval(timerInterval);
  }

  function goToHome() {
    goto('/');
  }

  function formatTime(timeMs: number): string {
    const minutes = Math.floor(timeMs / 60000);
    const seconds = Math.floor((timeMs % 60000) / 1000);
    const milliseconds = Math.floor((timeMs % 1000) / 10);
    return `${minutes}:${seconds.toString().padStart(2, '0')}.${milliseconds.toString().padStart(2, '0')}`;
  }

  function getAccuracy(): number {
    return totalCount > 0 ? Math.round((correctCount / totalCount) * 100) : 0;
  }

  // Auto-advance when correct answer is typed
  $: if (userInput && currentKana && gameStarted && !gameCompleted) {
    if (userInput.toLowerCase().trim() === currentKana.romaji.toLowerCase()) {
      setTimeout(() => {
        checkAnswer();
      }, 200); // Small delay to show the correct answer
    }
  }
</script>

<div class="nav-bar">
  <button class="home-button" on:click={goToHome}>Back to Home</button>
</div>

<main class="game-container">
  <h1>Kana Speed Typing Game</h1>
  
  {#if loading}
    <div class="loading">Loading kana data...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if !gameStarted}
    <!-- Game Setup Screen -->
    <div class="setup-screen">
      <div class="game-mode-selection">
        <h2>Choose Game Settings</h2>
        
        <!-- Kana Type Selection -->
        <div class="setting-group">
          <h3>Kana Type</h3>
          <div class="mode-buttons">
            <button 
              class="mode-button {gameMode === 'hiragana' ? 'active' : ''}"
              on:click={() => gameMode = 'hiragana'}
            >
              Hiragana Only
            </button>
            <button 
              class="mode-button {gameMode === 'katakana' ? 'active' : ''}"
              on:click={() => gameMode = 'katakana'}
            >
              Katakana Only
            </button>
            <button 
              class="mode-button {gameMode === 'mixed' ? 'active' : ''}"
              on:click={() => gameMode = 'mixed'}
            >
              Mixed (Both)
            </button>
          </div>
        </div>

        <!-- Sound Type Selection -->
        <div class="setting-group">
          <h3>Sound Types</h3>
          <div class="checkbox-buttons">
            <button 
              class="checkbox-button {soundTypes.includes('seion') ? 'active' : ''}"
              on:click={() => toggleSoundType('seion')}
            >
              <span class="checkbox-indicator">{soundTypes.includes('seion') ? '✓' : ''}</span>
              Seion (Basic)
              <div class="subtitle">あ か さ た な は ま や ら わ</div>
            </button>
            <button 
              class="checkbox-button {soundTypes.includes('dakuon') ? 'active' : ''}"
              on:click={() => toggleSoundType('dakuon')}
            >
              <span class="checkbox-indicator">{soundTypes.includes('dakuon') ? '✓' : ''}</span>
              Dakuon (Voiced)
              <div class="subtitle">が ざ だ ば</div>
            </button>
            <button 
              class="checkbox-button {soundTypes.includes('handakuon') ? 'active' : ''}"
              on:click={() => toggleSoundType('handakuon')}
            >
              <span class="checkbox-indicator">{soundTypes.includes('handakuon') ? '✓' : ''}</span>
              Handakuon (Semi-voiced)
              <div class="subtitle">ぱ ぴ ぷ ぺ ぽ</div>
            </button>
          </div>
        </div>

        <!-- Complexity Selection -->
        <div class="setting-group">
          <h3>Complexity Level</h3>
          <div class="mode-buttons">
            <button 
              class="mode-button {complexity === 'basic' ? 'active' : ''}"
              on:click={() => complexity = 'basic'}
            >
              Basic Characters
              <div class="subtitle">Single sounds (あ, か, etc.)</div>
            </button>
            <button 
              class="mode-button {complexity === 'combination' ? 'active' : ''}"
              on:click={() => complexity = 'combination'}
            >
              Combination Characters
              <div class="subtitle">ya/yu/yo combinations (きゃ, しゅ, etc.)</div>
            </button>
            <button 
              class="mode-button {complexity === 'mixed' ? 'active' : ''}"
              on:click={() => complexity = 'mixed'}
            >
              Mixed (Both)
            </button>
          </div>
        </div>

        <!-- Game Preview -->
        <div class="game-preview">
          <div class="preview-info">
            <strong>Game Preview:</strong> {getFilteredKanaCount()} characters selected
          </div>
          {#if bestTimes[generateGameKey()]}
            <div class="current-best">
              Current Best: {formatTime(bestTimes[generateGameKey()])}
            </div>
          {/if}
        </div>
      </div>
      
      <button 
        class="start-button" 
        on:click={startGame}
        disabled={getFilteredKanaCount() === 0}
      >
        Start Game
      </button>
      
      <div class="instructions">
        <h3>How to Play:</h3>
        <ul>
          <li>Select your preferred kana type, sound types, and complexity level</li>
          <li>Type the romaji (English letters) for each kana character shown</li>
          <li>The game auto-advances when you type correctly</li>
          <li>Try to complete all characters as fast as possible!</li>
          <li>Your best times are saved for each combination of settings</li>
        </ul>
      </div>
    </div>
  {:else if gameCompleted}
    <!-- Results Screen -->
    <div class="results-screen">
      <h2>Game Complete! 🎉</h2>
      
      <div class="final-stats">
        <div class="stat">
          <div class="stat-label">Time</div>
          <div class="stat-value">{formatTime(endTime - startTime)}</div>
        </div>
        <div class="stat">
          <div class="stat-label">Accuracy</div>
          <div class="stat-value">{getAccuracy()}%</div>
        </div>
        <div class="stat">
          <div class="stat-label">Characters</div>
          <div class="stat-value">{correctCount}/{totalCount}</div>
        </div>
      </div>
      
      {#if bestTimes[gameMode] === (endTime - startTime)}
        <div class="new-best">🏆 New Best Time!</div>
      {/if}
      
      <div class="result-actions">
        <button class="play-again-button" on:click={resetGame}>
          Play Again
        </button>
        <button class="home-button" on:click={goToHome}>
          Back to Home
        </button>
      </div>
    </div>
  {:else}
    <!-- Game Screen -->
    <div class="game-screen">
      <div class="game-header">
        <div class="timer">
          Time: {formatTime(elapsedTime)}
        </div>
        <div class="progress">
          {currentIndex + 1} / {totalCount}
        </div>
        <div class="accuracy">
          Accuracy: {getAccuracy()}%
        </div>
      </div>
      
      <div class="kana-display">
        <div class="current-kana">
          {currentKana?.character || ''}
        </div>
        <div class="kana-type-indicator">
          {currentKana?.kana_type || ''}
        </div>
      </div>
      
      <div class="input-section">
        <input 
          type="text" 
          bind:value={userInput}
          on:keydown={handleInput}
          placeholder="Type romaji here..."
          class="romaji-input"
          autocomplete="off"
        />
        <button class="submit-button" on:click={checkAnswer}>
          Submit
        </button>
      </div>
      
      <div class="game-actions">
        <button class="quit-button" on:click={resetGame}>
          Quit Game
        </button>
      </div>
    </div>
  {/if}
</main>
