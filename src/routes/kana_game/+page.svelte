<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { isCorrectRomaji } from '$lib/romaji';
  import './kana-game.css';

  interface KanaChar {
    character: string;
    romaji: string;
    kana_type: string;
    sound_type: string;
    complexity: string;
  }

  type SoundType = 'seion' | 'dakuon' | 'handakuon';
  type GameMode = 'hiragana' | 'katakana' | 'mixed';
  type Complexity = 'basic' | 'combination' | 'mixed';
  const SOUND_TYPE_ORDER: SoundType[] = ['seion', 'dakuon', 'handakuon'];

  interface TroubleKana {
    kana: KanaChar;
    misses: number;
    skipped: boolean;
  }

  let kanaData: KanaChar[] = [];
  let loading = true;
  let error = '';

  // Game settings
  let gameMode: GameMode = 'hiragana';
  let soundTypes: SoundType[] = ['seion'];
  let complexity: Complexity = 'basic';
  let gameStarted = false;
  let gameCompleted = false;

  // Game state
  let currentKana: KanaChar | null = null;
  let userInput = '';
  let currentIndex = 0;
  let gameKana: KanaChar[] = [];
  let correctCount = 0;
  let mistakeCount = 0;
  let skippedCount = 0;
  let totalCount = 0;
  let troubleKana: TroubleKana[] = [];
  let feedback: { text: string; kind: 'correct' | 'incorrect' | 'skipped' } | null = null;
  let inputElement: HTMLInputElement | null = null;

  // Timing
  let startTime = 0;
  let endTime = 0;
  let elapsedTime = 0;
  let timerInterval: ReturnType<typeof setInterval> | undefined;

  // Best times keyed by game settings (see generateGameKey)
  let bestTimes: Record<string, number> = {};
  let isNewBest = false;
  // A practice round replays only the characters missed in the last game
  // and doesn't count toward best times.
  let practiceRound = false;

  $: selectedCount = filterKana(kanaData, gameMode, soundTypes, complexity).length;
  $: savedRecords = Object.entries(bestTimes)
    .map(([key, time]) => ({ key, time, settings: parseGameKey(key) }))
    .filter((record) => record.settings !== null)
    .sort((a, b) => a.key.localeCompare(b.key));
  $: gameKey = generateGameKey(gameMode, soundTypes, complexity);
  $: attempts = correctCount + mistakeCount + skippedCount;
  $: accuracyText = attempts > 0 ? `${Math.round((correctCount / attempts) * 100)}%` : '—';

  onMount(async () => {
    await loadKanaData();
    await loadBestTimes();
  });

  // Don't leave the timer running if the user navigates away mid-game
  onDestroy(stopTimer);

  async function loadKanaData() {
    try {
      loading = true;
      error = '';
      kanaData = await invoke('get_kana_data');
    } catch (err) {
      error = `Failed to load kana data: ${err}`;
      console.error('Error loading kana data:', err);
    } finally {
      loading = false;
    }
  }

  async function loadBestTimes() {
    try {
      bestTimes = await invoke('get_best_times');
    } catch (err) {
      console.error('Failed to load best times:', err);
    }
  }

  function filterKana(
    data: KanaChar[],
    mode: GameMode,
    types: SoundType[],
    level: Complexity
  ): KanaChar[] {
    return data.filter(
      (k) =>
        (mode === 'mixed' || k.kana_type === mode) &&
        types.includes(k.sound_type as SoundType) &&
        (level === 'mixed' || k.complexity === level)
    );
  }

  // e.g. "hiragana-seion-basic" or "katakana-seion+dakuon-mixed". Every
  // combination of sound types gets its own key so best times for
  // different character sets don't overwrite each other.
  function generateGameKey(
    mode: GameMode,
    types: SoundType[],
    level: Complexity
  ): string {
    const soundKey = SOUND_TYPE_ORDER.filter((t) => types.includes(t)).join('+');
    return `${mode}-${soundKey}-${level}`;
  }

  interface GameSettings {
    mode: GameMode;
    types: SoundType[];
    level: Complexity;
  }

  // Inverse of generateGameKey. Keys from older versions used "mixed" for
  // any combination of sound types; those can't be restored and are skipped.
  function parseGameKey(key: string): GameSettings | null {
    const [mode, soundKey, level] = key.split('-');
    const types = (soundKey ?? '').split('+') as SoundType[];
    const validMode = mode === 'hiragana' || mode === 'katakana' || mode === 'mixed';
    const validLevel = level === 'basic' || level === 'combination' || level === 'mixed';
    const validTypes = types.length > 0 && types.every((t) => SOUND_TYPE_ORDER.includes(t));
    if (!validMode || !validLevel || !validTypes) return null;
    return { mode, types, level };
  }

  const capitalize = (word: string) => word.charAt(0).toUpperCase() + word.slice(1);

  function describeSettings(settings: GameSettings): string {
    const mode = settings.mode === 'mixed' ? 'Hiragana + Katakana' : capitalize(settings.mode);
    const types = settings.types.map(capitalize).join(' + ');
    const level = settings.level === 'mixed' ? 'all' : settings.level;
    return `${mode} · ${types} · ${level}`;
  }

  function useSettings(settings: GameSettings) {
    gameMode = settings.mode;
    soundTypes = [...settings.types];
    complexity = settings.level;
  }

  function shuffleArray<T>(array: T[]): T[] {
    const shuffled = [...array];
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    return shuffled;
  }

  function stopTimer() {
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = undefined;
    }
  }

  async function startGame() {
    practiceRound = false;
    await beginRound(filterKana(kanaData, gameMode, soundTypes, complexity));
  }

  async function practiceMissed() {
    const missed = troubleKana.map((t) => t.kana);
    practiceRound = true;
    await beginRound(missed);
  }

  async function beginRound(kana: KanaChar[]) {
    if (kana.length === 0) return;

    gameKana = shuffleArray(kana);
    currentIndex = 0;
    correctCount = 0;
    mistakeCount = 0;
    skippedCount = 0;
    totalCount = gameKana.length;
    troubleKana = [];
    feedback = null;
    isNewBest = false;
    userInput = '';
    gameStarted = true;
    gameCompleted = false;
    currentKana = gameKana[0];

    stopTimer();
    startTime = Date.now();
    elapsedTime = 0;
    timerInterval = setInterval(() => {
      elapsedTime = Date.now() - startTime;
    }, 50);

    await tick();
    inputElement?.focus();
  }

  // Advance as soon as the typed answer is correct. This happens
  // synchronously, so there is no delayed check that could fire after the
  // next kana is shown and wipe what the user is typing.
  function handleInput() {
    if (!currentKana || gameCompleted) return;
    if (isCorrectRomaji(userInput, currentKana.character, currentKana.romaji)) {
      feedback = { text: `${currentKana.character} = ${currentKana.romaji}`, kind: 'correct' };
      correctCount++;
      advance();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      submitAnswer();
    }
  }

  // Enter / Submit with a wrong answer counts as a mistake.
  function submitAnswer() {
    if (!currentKana || gameCompleted) return;
    const answer = userInput.trim();
    if (answer === '') return;

    if (isCorrectRomaji(answer, currentKana.character, currentKana.romaji)) {
      handleInput();
      return;
    }

    mistakeCount++;
    noteTrouble(currentKana, false);
    feedback = { text: `"${answer}" is not ${currentKana.character}`, kind: 'incorrect' };
    userInput = '';
    inputElement?.focus();
  }

  function skipKana() {
    if (!currentKana || gameCompleted) return;
    skippedCount++;
    noteTrouble(currentKana, true);
    feedback = { text: `${currentKana.character} = ${currentKana.romaji}`, kind: 'skipped' };
    advance();
    inputElement?.focus();
  }

  function noteTrouble(kana: KanaChar, skipped: boolean) {
    const existing = troubleKana.find((t) => t.kana === kana);
    if (existing) {
      if (skipped) existing.skipped = true;
      else existing.misses++;
      troubleKana = troubleKana;
    } else {
      troubleKana = [...troubleKana, { kana, misses: skipped ? 0 : 1, skipped }];
    }
  }

  function advance() {
    userInput = '';
    currentIndex++;
    if (currentIndex >= gameKana.length) {
      endGame();
    } else {
      currentKana = gameKana[currentIndex];
    }
  }

  async function endGame() {
    gameCompleted = true;
    endTime = Date.now();
    stopTimer();
    elapsedTime = endTime - startTime;

    // Skipping is faster than typing, so only clean runs set best times;
    // practice rounds use a different set of characters.
    if (skippedCount > 0 || practiceRound) return;

    const finalTime = endTime - startTime;
    const key = gameKey;
    const currentBest = bestTimes[key];
    if (currentBest === undefined || finalTime < currentBest) {
      isNewBest = true;
      bestTimes = { ...bestTimes, [key]: finalTime };
      try {
        await invoke('save_best_time', { gameKey: key, timeMs: finalTime });
      } catch (err) {
        console.error('Failed to save best time:', err);
      }
    }
  }

  function toggleSoundType(type: SoundType) {
    if (soundTypes.includes(type)) {
      soundTypes = soundTypes.filter((t) => t !== type);
    } else {
      soundTypes = [...soundTypes, type];
    }
    // Ensure at least one sound type is selected
    if (soundTypes.length === 0) {
      soundTypes = ['seion'];
    }
  }

  function resetGame() {
    stopTimer();
    gameStarted = false;
    gameCompleted = false;
    currentKana = null;
    userInput = '';
    currentIndex = 0;
    correctCount = 0;
    mistakeCount = 0;
    skippedCount = 0;
    totalCount = 0;
    troubleKana = [];
    feedback = null;
    elapsedTime = 0;
  }

  function goToHome() {
    goto('/');
  }

  function formatTime(timeMs: number): string {
    const minutes = Math.floor(timeMs / 60000);
    const seconds = Math.floor((timeMs % 60000) / 1000);
    const centiseconds = Math.floor((timeMs % 1000) / 10);
    return `${minutes}:${seconds.toString().padStart(2, '0')}.${centiseconds.toString().padStart(2, '0')}`;
  }
</script>

<div class="nav-bar">
  <button class="home-button" on:click={goToHome}>Back to Home</button>
</div>

<main class="game-container">
  <h1>Kana Speed Typing Game</h1>
  {#if gameStarted && practiceRound}
    <div class="practice-badge">Practice round: characters you missed</div>
  {/if}

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
              on:click={() => (gameMode = 'hiragana')}
            >
              Hiragana Only
            </button>
            <button
              class="mode-button {gameMode === 'katakana' ? 'active' : ''}"
              on:click={() => (gameMode = 'katakana')}
            >
              Katakana Only
            </button>
            <button
              class="mode-button {gameMode === 'mixed' ? 'active' : ''}"
              on:click={() => (gameMode = 'mixed')}
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
              on:click={() => (complexity = 'basic')}
            >
              Basic Characters
              <div class="subtitle">Single sounds (あ, か, etc.)</div>
            </button>
            <button
              class="mode-button {complexity === 'combination' ? 'active' : ''}"
              on:click={() => (complexity = 'combination')}
            >
              Combination Characters
              <div class="subtitle">ya/yu/yo combinations (きゃ, しゅ, etc.)</div>
            </button>
            <button
              class="mode-button {complexity === 'mixed' ? 'active' : ''}"
              on:click={() => (complexity = 'mixed')}
            >
              Mixed (Both)
            </button>
          </div>
        </div>

        <!-- Game Preview -->
        <div class="game-preview">
          <div class="preview-info">
            <strong>Game Preview:</strong> {selectedCount} characters selected
          </div>
          {#if bestTimes[gameKey] !== undefined}
            <div class="current-best">
              Current Best: {formatTime(bestTimes[gameKey])}
            </div>
          {/if}
        </div>

        {#if savedRecords.length > 0}
          <details class="best-times">
            <summary>Your best times ({savedRecords.length})</summary>
            <ul>
              {#each savedRecords as record}
                <li>
                  <button
                    class="best-time-row {record.key === gameKey ? 'current' : ''}"
                    on:click={() => record.settings && useSettings(record.settings)}
                    title="Use these settings"
                  >
                    <span>{record.settings ? describeSettings(record.settings) : record.key}</span>
                    <span class="best-time-value">{formatTime(record.time)}</span>
                  </button>
                </li>
              {/each}
            </ul>
          </details>
        {/if}
      </div>

      <button class="start-button" on:click={startGame} disabled={selectedCount === 0}>
        Start Game
      </button>

      <div class="instructions">
        <h3>How to Play:</h3>
        <ul>
          <li>Select your preferred kana type, sound types, and complexity level</li>
          <li>Type the romaji (English letters) for each kana character shown</li>
          <li>The game advances as soon as you type a correct answer</li>
          <li>Common alternative spellings are accepted (si/shi, tu/tsu, hu/fu, ...)</li>
          <li>Press Enter to check a wrong answer, or Skip to reveal it</li>
          <li>Try to complete all characters as fast as possible!</li>
          <li>Best times are saved for each combination of settings (runs with skips don't count)</li>
        </ul>
      </div>
    </div>
  {:else if gameCompleted}
    <!-- Results Screen -->
    <div class="results-screen">
      <h2>{practiceRound ? 'Practice Complete!' : 'Game Complete! 🎉'}</h2>

      <div class="final-stats">
        <div class="stat">
          <div class="stat-label">Time</div>
          <div class="stat-value">{formatTime(endTime - startTime)}</div>
        </div>
        <div class="stat">
          <div class="stat-label">Accuracy</div>
          <div class="stat-value">{accuracyText}</div>
        </div>
        <div class="stat">
          <div class="stat-label">Mistakes</div>
          <div class="stat-value">{mistakeCount}</div>
        </div>
        <div class="stat">
          <div class="stat-label">Skipped</div>
          <div class="stat-value">{skippedCount}/{totalCount}</div>
        </div>
      </div>

      {#if isNewBest}
        <div class="new-best">🏆 New Best Time!</div>
      {:else if practiceRound}
        <div class="best-note">Practice rounds don't count toward best times.</div>
      {:else if skippedCount > 0}
        <div class="best-note">Runs with skipped characters don't count toward best times.</div>
      {/if}

      {#if troubleKana.length > 0}
        <div class="trouble-section">
          <h3>Characters to practice</h3>
          <div class="trouble-list">
            {#each troubleKana as trouble}
              <div class="trouble-item" title={trouble.skipped ? 'Skipped' : `${trouble.misses} wrong`}>
                <span class="trouble-kana">{trouble.kana.character}</span>
                <span class="trouble-romaji">{trouble.kana.romaji}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <div class="result-actions">
        {#if troubleKana.length > 0}
          <button class="play-again-button" on:click={practiceMissed}>
            Practice these ({troubleKana.length})
          </button>
        {/if}
        <button class="play-again-button {troubleKana.length > 0 ? 'secondary' : ''}" on:click={startGame}>
          {practiceRound ? 'Full Game' : 'Play Again'}
        </button>
        <button class="home-button" on:click={resetGame}>Change Settings</button>
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
          Accuracy: {accuracyText}
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
          bind:this={inputElement}
          bind:value={userInput}
          on:input={handleInput}
          on:keydown={handleKeydown}
          placeholder="Type romaji here..."
          class="romaji-input"
          autocomplete="off"
          autocapitalize="off"
          spellcheck="false"
        />
        <button class="submit-button" on:click={submitAnswer}>Submit</button>
        <button class="skip-button" on:click={skipKana}>Skip</button>
      </div>

      <div class="feedback {feedback ? feedback.kind : ''}" aria-live="polite">
        {#if feedback}
          {#if feedback.kind === 'correct'}✓{:else if feedback.kind === 'incorrect'}✗{:else}→{/if}
          {feedback.text}
        {/if}
      </div>

      <div class="game-actions">
        <button class="quit-button" on:click={resetGame}>Quit Game</button>
      </div>
    </div>
  {/if}
</main>
