<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import './kana-game.css';

  interface KanaChar {
    character: string;
    romaji: string;
    kana_type: string;
  }

  let kanaData: KanaChar[] = [];
  let loading = true;
  let error = '';
  
  // Game settings
  let gameMode: 'hiragana' | 'katakana' | 'mixed' = 'hiragana';
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
    hiragana: localStorage.getItem('best-hiragana-time') ? parseInt(localStorage.getItem('best-hiragana-time')!) : null,
    katakana: localStorage.getItem('best-katakana-time') ? parseInt(localStorage.getItem('best-katakana-time')!) : null,
    mixed: localStorage.getItem('best-mixed-time') ? parseInt(localStorage.getItem('best-mixed-time')!) : null
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
    // Filter kana based on game mode
    let filteredKana = [];
    if (gameMode === 'mixed') {
      filteredKana = [...kanaData];
    } else {
      filteredKana = kanaData.filter(k => k.kana_type === gameMode);
    }
    
    // Shuffle the kana
    gameKana = shuffleArray([...filteredKana]);
    
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
    
    // Check if this is a new best time
    const currentBest = bestTimes[gameMode];
    if (!currentBest || finalTime < currentBest) {
      bestTimes[gameMode] = finalTime;
      localStorage.setItem(`best-${gameMode}-time`, finalTime.toString());
    }
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
        <h2>Choose Game Mode</h2>
        <div class="mode-buttons">
          <button 
            class="mode-button {gameMode === 'hiragana' ? 'active' : ''}"
            on:click={() => gameMode = 'hiragana'}
          >
            Hiragana Only
            {#if bestTimes.hiragana}
              <div class="best-time">Best: {formatTime(bestTimes.hiragana)}</div>
            {/if}
          </button>
          <button 
            class="mode-button {gameMode === 'katakana' ? 'active' : ''}"
            on:click={() => gameMode = 'katakana'}
          >
            Katakana Only
            {#if bestTimes.katakana}
              <div class="best-time">Best: {formatTime(bestTimes.katakana)}</div>
            {/if}
          </button>
          <button 
            class="mode-button {gameMode === 'mixed' ? 'active' : ''}"
            on:click={() => gameMode = 'mixed'}
          >
            Mixed (Both)
            {#if bestTimes.mixed}
              <div class="best-time">Best: {formatTime(bestTimes.mixed)}</div>
            {/if}
          </button>
        </div>
      </div>
      
      <button class="start-button" on:click={startGame}>
        Start Game
      </button>
      
      <div class="instructions">
        <h3>How to Play:</h3>
        <ul>
          <li>Type the romaji (English letters) for each kana character shown</li>
          <li>Press Enter or the input will auto-advance when correct</li>
          <li>Try to complete all characters as fast as possible!</li>
          <li>Your best times are saved automatically</li>
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
