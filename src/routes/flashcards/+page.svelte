<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { type KanjiLookup, getGradeLabel, getJlptLabel } from '$lib/kanji';
  import { type SrsCard } from '$lib/srs';
  import { matchReading } from '$lib/answers';
  import { getSessionLimits } from '$lib/preferences';
  import './flashcards.css';

  type KanjiCard = KanjiLookup;

  interface ReviewSession {
    cards_due: SrsCard[];
    cards_new: SrsCard[];
  }

  interface StudyStats {
    total_cards: number;
    cards_due: number;
    cards_new: number;
    cards_learning: number;
    cards_mature: number;
    daily_streak: number;
    total_reviews: number;
    accuracy: number;
    week_dates: string[]; // last 7 days as YYYY-MM-DD, oldest first (last = today)
    reviews_this_week: number[];
    accuracy_this_week: number[];
    forecast: number[]; // reviews coming due on each of the next 7 days (0 = tomorrow)
  }

  // A card graded "Again" comes back after this many other cards
  const REQUEUE_GAP = 3;
  const TYPE_ANSWERS_KEY = 'flashcards.typeAnswers';

  // App state
  let loading = true;
  let error = '';
  let currentView = 'dashboard'; // dashboard, study, stats
  let studyStats: StudyStats | null = null;
  let notice = '';

  // Study session
  let queue: SrsCard[] = []; // cards still to review, in order
  let sessionReviews = 0;
  let sessionCorrect = 0;
  let missedCharacters: string[] = []; // graded Again or Hard this session
  let sessionStartTime = 0;
  let sessionEndTime = 0;
  let sessionComplete = false;
  let currentCard: SrsCard | null = null;
  let currentKanjiData: KanjiCard | null = null;
  let currentSvg = '';
  let reviewType: 'meaning' | 'reading' = 'meaning';
  let showAnswer = false;
  let submitting = false; // blocks double-clicks from reviewing a card twice
  let cardRequest = 0; // ignores kanji data that arrives after moving on

  // Typed answers for reading questions
  let typeAnswers = false;
  let typedAnswer = '';
  let matchedReading: string | null = null;
  let answerInput: HTMLInputElement | null = null;

  $: remainingCards = queue.length;
  $: sessionProgress =
    sessionReviews + remainingCards > 0 ? sessionReviews / (sessionReviews + remainingCards) : 0;
  $: sessionAccuracy =
    sessionReviews > 0 ? `${((sessionCorrect / sessionReviews) * 100).toFixed(1)}%` : 'N/A';
  $: askTypedReading = typeAnswers && reviewType === 'reading';
  // Grade suggested by a typed answer: Good if it matched, otherwise Again
  $: suggestedGrade = askTypedReading && showAnswer ? (matchedReading ? 3 : 1) : 3;

  onMount(async () => {
    try {
      typeAnswers = localStorage.getItem(TYPE_ANSWERS_KEY) === 'true';
    } catch {
      typeAnswers = false;
    }
    await loadStudyStats();
    loading = false;
  });

  async function setTypeAnswers(enabled: boolean) {
    typeAnswers = enabled;
    try {
      localStorage.setItem(TYPE_ANSWERS_KEY, String(enabled));
    } catch {
      // Not persisted; the setting still applies to this visit
    }
    if (enabled && reviewType === 'reading' && !showAnswer) {
      await tick();
      answerInput?.focus();
    }
  }

  async function loadStudyStats() {
    try {
      studyStats = await invoke('get_study_stats');
    } catch (err) {
      error = `Failed to load study stats: ${err}`;
      console.error('Error loading study stats:', err);
    }
  }

  async function startReviewSession(sessionType: 'due' | 'new' | 'mixed') {
    try {
      loading = true;
      notice = '';
      const { dueLimit, newLimit } = getSessionLimits();
      const session: ReviewSession = await invoke('start_review_session', {
        sessionType,
        dueLimit,
        newLimit
      });
      const cards = [...session.cards_due, ...session.cards_new];
      if (cards.length > 0) {
        queue = cards;
        sessionReviews = 0;
        sessionCorrect = 0;
        missedCharacters = [];
        sessionStartTime = Date.now();
        sessionEndTime = 0;
        sessionComplete = false;
        currentView = 'study';
        await nextCard();
      } else {
        // Not fatal: stay on the dashboard and let the user add cards
        notice = 'No cards available for review. Add some new cards to get started.';
        await loadStudyStats();
      }
    } catch (err) {
      error = `Failed to start review session: ${err}`;
      console.error('Error starting review session:', err);
    } finally {
      loading = false;
    }
  }

  async function nextCard() {
    const request = ++cardRequest;
    showAnswer = false;
    typedAnswer = '';
    matchedReading = null;
    // Clear the previous card's data so its answer can't be shown for this one
    currentKanjiData = null;
    currentSvg = '';

    if (queue.length === 0) {
      // Session complete: show the summary and refresh the stats for when
      // the user goes back to the dashboard
      currentCard = null;
      sessionComplete = true;
      sessionEndTime = Date.now();
      await loadStudyStats();
      return;
    }

    currentCard = queue[0];
    const character = currentCard.character;
    try {
      const [kanji, svg] = await Promise.all([
        invoke<KanjiCard>('get_kanji', { character }),
        invoke<string>('get_kanji_svg', { character }).catch(() => '')
      ]);
      if (request !== cardRequest) return;
      // Randomly choose review type (60% meaning, 40% reading), but only ask
      // for something the dictionary has an answer for
      const hasReadings = Boolean(kanji.onyomi || kanji.kunyomi);
      const hasMeanings = kanji.meanings.length > 0;
      if (hasReadings && hasMeanings) {
        reviewType = Math.random() < 0.6 ? 'meaning' : 'reading';
      } else {
        reviewType = hasReadings ? 'reading' : 'meaning';
      }
      currentKanjiData = kanji;
      currentSvg = svg;
      if (typeAnswers && reviewType === 'reading') {
        await tick();
        answerInput?.focus();
      }
    } catch (err) {
      if (request !== cardRequest) return;
      console.error('Error loading kanji data:', err);
      error = `Failed to load kanji data for ${character}: ${err}`;
    }
  }

  function revealAnswer() {
    if (!currentKanjiData || showAnswer) return;
    if (askTypedReading) {
      matchedReading = matchReading(typedAnswer, currentKanjiData.onyomi, currentKanjiData.kunyomi);
    }
    showAnswer = true;
  }

  async function submitAnswer(grade: number) {
    if (!currentCard || !showAnswer || submitting) return;

    submitting = true;
    try {
      const updatedCard: SrsCard = await invoke('submit_card_review', {
        cardId: currentCard.id,
        grade,
        reviewType
      });

      sessionReviews += 1;
      if (grade >= 3) {
        sessionCorrect += 1;
      } else if (!missedCharacters.includes(currentCard.character)) {
        missedCharacters = [...missedCharacters, currentCard.character];
      }

      // Take the card off the front of the queue. A card graded "Again"
      // comes back a few cards later so it's practiced before the session
      // ends.
      const rest = queue.slice(1);
      if (grade === 1) {
        const position = Math.min(REQUEUE_GAP, rest.length);
        rest.splice(position, 0, updatedCard);
      }
      queue = rest;

      await nextCard();
    } catch (err) {
      error = `Failed to submit answer: ${err}`;
      console.error('Error submitting answer:', err);
    } finally {
      submitting = false;
    }
  }

  // Space/Enter reveal the answer and then accept the suggested grade;
  // 1-4 grade the card.
  function handleKeydown(event: KeyboardEvent) {
    if (currentView !== 'study' || sessionComplete || !currentKanjiData) return;
    if (event.ctrlKey || event.metaKey || event.altKey || event.repeat) return;
    const typing = event.target instanceof HTMLInputElement;

    if (!showAnswer) {
      if (event.key === 'Enter' || (event.key === ' ' && !typing)) {
        event.preventDefault();
        revealAnswer();
      }
      return;
    }

    if (['1', '2', '3', '4'].includes(event.key)) {
      event.preventDefault();
      submitAnswer(Number(event.key));
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      submitAnswer(suggestedGrade);
    }
  }

  async function addNewCards(count: number) {
    try {
      const added: number = await invoke('add_new_cards', { count });
      notice = added === 0
        ? 'No more kanji are available to add.'
        : added < count
          ? `Added ${added} new cards (no more kanji available).`
          : `Added ${added} new cards.`;
      await loadStudyStats();
    } catch (err) {
      error = `Failed to add new cards: ${err}`;
      console.error('Error adding new cards:', err);
    }
  }

  function dismissError() {
    error = '';
    goToDashboard();
  }

  // Reload the stats too: reviews done in an unfinished session already
  // changed the counts.
  function goToDashboard() {
    cardRequest++;
    currentView = 'dashboard';
    loadStudyStats();
  }

  function goToHome() {
    goto('/');
  }

  function formatTimeSpent(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    }
    return `${seconds}s`;
  }

  function plural(count: number, word: string): string {
    return `${count} ${count === 1 ? word : word + 's'}`;
  }

  // "Tomorrow", then weekday names for the forecast columns
  function forecastLabel(daysFromToday: number): string {
    if (daysFromToday === 1) return 'Tomorrow';
    const date = new Date();
    date.setDate(date.getDate() + daysFromToday);
    return date.toLocaleDateString('en-US', { weekday: 'short' });
  }

  // Label a "YYYY-MM-DD" date from the backend with its weekday name.
  function weekdayLabel(dateStr: string | undefined, style: 'short' | 'long'): string {
    if (!dateStr) return '';
    const [year, month, day] = dateStr.split('-').map(Number);
    const date = new Date(year, month - 1, day);
    const today = new Date();
    if (date.toDateString() === today.toDateString()) return 'Today';
    return date.toLocaleDateString('en-US', { weekday: style });
  }
</script>

<div class="nav-bar">
  <button class="home-button" on:click={goToHome}>Back to Home</button>
  {#if currentView !== 'dashboard'}
    <button class="back-button" on:click={goToDashboard}>Dashboard</button>
  {/if}
</div>

<main class="flashcards-container">
  {#if loading}
    <div class="loading">Loading...</div>
  {:else if error}
    <div class="error">
      {error}
      <div>
        <button class="back-button" on:click={dismissError}>Back to Dashboard</button>
      </div>
    </div>
  {:else if currentView === 'dashboard'}
    <!-- Dashboard View -->
    <div class="dashboard">
      <h1>Kanji Flashcards</h1>

      {#if notice}
        <div class="notice">{notice}</div>
      {/if}
      
      {#if studyStats}
        <!-- Study Stats Overview -->
        <div class="stats-overview">
          <div class="stat-card due">
            <div class="stat-number">{studyStats.cards_due}</div>
            <div class="stat-label">Due Today</div>
          </div>
          <div class="stat-card new">
            <div class="stat-number">{studyStats.cards_new}</div>
            <div class="stat-label">New Cards</div>
          </div>
          <div class="stat-card learning">
            <div class="stat-number">{studyStats.cards_learning}</div>
            <div class="stat-label">Learning</div>
          </div>
          <div class="stat-card mature">
            <div class="stat-number">{studyStats.cards_mature}</div>
            <div class="stat-label">Mature</div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="action-buttons">
          {#if studyStats.cards_due > 0}
            <button class="study-button due" on:click={() => startReviewSession('due')}>
              Study Due Cards ({studyStats.cards_due})
            </button>
          {/if}
          {#if studyStats.cards_new > 0}
            <button class="study-button new" on:click={() => startReviewSession('new')}>
              Learn New Cards ({studyStats.cards_new})
            </button>
          {/if}
          {#if studyStats.cards_due > 0 || studyStats.cards_new > 0}
            <button class="study-button mixed" on:click={() => startReviewSession('mixed')}>
              Mixed Review ({studyStats.cards_due + studyStats.cards_new})
            </button>
          {/if}
        </div>

        <!-- Add New Cards -->
        <div class="add-cards-section">
          <h3>Add New Cards</h3>
          <div class="add-cards-buttons">
            <button class="add-button" on:click={() => addNewCards(5)}>+5 Cards</button>
            <button class="add-button" on:click={() => addNewCards(10)}>+10 Cards</button>
            <button class="add-button" on:click={() => addNewCards(20)}>+20 Cards</button>
          </div>
          <p class="add-cards-hint">
            New cards are picked by school grade and frequency. You can also add specific kanji from the
            <a href="/kanji_view">Kanji Database</a>
            {#if studyStats.total_cards > 0}
              or <a href="/kanji_view?tab=deck">browse your {studyStats.total_cards}
                {studyStats.total_cards === 1 ? 'flashcard' : 'flashcards'}</a>
            {/if}.
          </p>
        </div>

        <!-- Quick Stats -->
        <div class="quick-stats">
          <div class="quick-stat">
            <span class="label">Daily Streak:</span>
            <span class="value">{plural(studyStats.daily_streak, 'day')}</span>
          </div>
          <div class="quick-stat">
            <span class="label">Total Reviews:</span>
            <span class="value">{studyStats.total_reviews.toLocaleString()}</span>
          </div>
          <div class="quick-stat">
            <span class="label">Accuracy:</span>
            <span class="value">{(studyStats.accuracy * 100).toFixed(1)}%</span>
          </div>
        </div>

        <!-- Upcoming reviews -->
        {#if studyStats.forecast}
          {@const forecastTotal = studyStats.forecast.reduce((sum, n) => sum + n, 0)}
          {@const forecastMax = Math.max(...studyStats.forecast, 1)}
          <div class="forecast">
            <h3>Upcoming Reviews</h3>
            {#if forecastTotal === 0}
              <p class="forecast-empty">
                {studyStats.total_cards - studyStats.cards_new > 0
                  ? 'Nothing else comes due in the next 7 days.'
                  : 'Reviews will show up here once you start studying.'}
              </p>
            {:else}
              <div class="forecast-days">
                {#each studyStats.forecast as count, i}
                  <div class="forecast-day" title="{plural(count, 'review')}">
                    <div class="forecast-count">{count}</div>
                    <div class="forecast-bar-track">
                      <div class="forecast-bar" style="height: {(count / forecastMax) * 100}%"></div>
                    </div>
                    <div class="forecast-label">{forecastLabel(i + 1)}</div>
                  </div>
                {/each}
              </div>
              <p class="forecast-total">{plural(forecastTotal, 'review')} in the next 7 days</p>
            {/if}
          </div>
        {/if}

        <!-- Weekly Progress Chart -->
        {#if studyStats.reviews_this_week.length > 0}
          <div class="progress-chart">
            <h3>This Week's Progress</h3>
            <div class="chart-container">
              <div class="chart-bars">
                {#each studyStats.reviews_this_week as reviews, i}
                  <div class="chart-day">
                    <div class="bar-container">
                      <div 
                        class="bar reviews" 
                        style="height: {Math.max(4, (reviews / Math.max(...studyStats.reviews_this_week, 1)) * 100)}px"
                        title="{reviews} reviews"
                      ></div>
                    </div>
                    <div class="day-label">
                      {weekdayLabel(studyStats.week_dates[i], 'short')}
                    </div>
                    <div class="day-stats">
                      <div class="reviews-count">{reviews}</div>
                      <div class="accuracy-percent">{(studyStats.accuracy_this_week[i] * 100).toFixed(0)}%</div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          </div>
        {/if}

        <!-- Detailed Stats Button -->
        <button class="stats-button" on:click={() => currentView = 'stats'}>
          View Detailed Statistics
        </button>
      {/if}
    </div>
  {:else if currentView === 'study'}
    <!-- Study View -->
    <div class="study-session">
      {#if !sessionComplete && currentCard && currentKanjiData}
        <div class="session-header">
          <div class="session-progress">
            <div class="progress-info">
              {plural(remainingCards, 'card')} left
            </div>
            <div class="session-stats">
              Reviews: {sessionReviews} | Accuracy: {sessionAccuracy}
            </div>
          </div>
          <div class="progress-track" aria-hidden="true">
            <div class="progress-fill" style="width: {sessionProgress * 100}%"></div>
          </div>
          <label class="type-answers-toggle">
            <input
              type="checkbox"
              checked={typeAnswers}
              on:change={(e) => setTypeAnswers(e.currentTarget.checked)}
            />
            Type readings
          </label>
        </div>

        <div class="study-card">
          <div class="card-front">
            <div class="kanji-character">{currentCard.character}</div>

            {#if reviewType === 'meaning'}
              <div class="question">What does this kanji mean?</div>
            {:else}
              <div class="question">How do you read this kanji?</div>
            {/if}

            {#if !showAnswer}
              {#if askTypedReading}
                <form class="typed-answer" on:submit|preventDefault={revealAnswer}>
                  <input
                    bind:this={answerInput}
                    bind:value={typedAnswer}
                    class="typed-answer-input"
                    placeholder="Type a reading (romaji or kana)"
                    autocomplete="off"
                    autocapitalize="off"
                    spellcheck="false"
                  />
                  <button type="submit" class="show-answer-button">Check</button>
                </form>
              {:else}
                <button class="show-answer-button" on:click={revealAnswer}>
                  Show Answer
                </button>
              {/if}
              <div class="shortcut-hint">
                {askTypedReading ? 'Enter' : 'Space'} to reveal · 1–4 to grade
              </div>
            {:else}
              <div class="answer-section">
                {#if askTypedReading}
                  <div class="typed-result {matchedReading ? 'correct' : 'incorrect'}">
                    {#if matchedReading}
                      ✓ "{typedAnswer.trim()}" matches {matchedReading}
                    {:else if typedAnswer.trim()}
                      ✗ "{typedAnswer.trim()}" isn't one of the readings
                    {:else}
                      No answer typed
                    {/if}
                  </div>
                {/if}

                <div class="answer-layout">
                  <div class="answer">
                    {#if reviewType === 'meaning'}
                      <h4>Meanings:</h4>
                      <div class="meanings-list">
                        {#each currentKanjiData.meanings as meaning}
                          <span class="meaning-tag">{meaning}</span>
                        {/each}
                      </div>
                    {:else}
                      <h4>Readings:</h4>
                      {#if currentKanjiData.onyomi}
                        <div class="reading-item">
                          <span class="reading-label">On'yomi:</span> {currentKanjiData.onyomi}
                        </div>
                      {/if}
                      {#if currentKanjiData.kunyomi}
                        <div class="reading-item">
                          <span class="reading-label">Kun'yomi:</span> {currentKanjiData.kunyomi}
                        </div>
                      {/if}
                    {/if}
                  </div>

                  {#if currentSvg}
                    <!-- Stroke order; animated diagrams draw themselves when shown -->
                    <div class="answer-diagram" title="Stroke order">
                      {@html currentSvg}
                    </div>
                  {/if}
                </div>

                <!-- Additional Info -->
                <div class="additional-info">
                  <div class="info-item">
                    <span class="info-label">Strokes:</span> {currentKanjiData.stroke_count}
                  </div>
                  {#if currentKanjiData.grade > 0}
                    <div class="info-item">
                      <span class="info-label">Grade:</span> {getGradeLabel(currentKanjiData.grade)}
                    </div>
                  {/if}
                  {#if currentKanjiData.jlpt_level > 0}
                    <div class="info-item">
                      <span class="info-label">JLPT:</span> {getJlptLabel(currentKanjiData.jlpt_level)}
                    </div>
                  {/if}
                </div>

                <!-- Grading Buttons -->
                <div class="grade-buttons">
                  <button class="grade-button fail {suggestedGrade === 1 ? 'suggested' : ''}" disabled={submitting} on:click={() => submitAnswer(1)}>
                    <span class="grade-key">1</span> Again<br><span class="grade-desc">Complete blackout</span>
                  </button>
                  <button class="grade-button hard" disabled={submitting} on:click={() => submitAnswer(2)}>
                    <span class="grade-key">2</span> Hard<br><span class="grade-desc">Incorrect but remembered</span>
                  </button>
                  <button class="grade-button good {suggestedGrade === 3 ? 'suggested' : ''}" disabled={submitting} on:click={() => submitAnswer(3)}>
                    <span class="grade-key">3</span> Good<br><span class="grade-desc">Correct with effort</span>
                  </button>
                  <button class="grade-button easy" disabled={submitting} on:click={() => submitAnswer(4)}>
                    <span class="grade-key">4</span> Easy<br><span class="grade-desc">Perfect recall</span>
                  </button>
                </div>
                <div class="shortcut-hint">
                  Space/Enter: {suggestedGrade === 1 ? 'Again' : 'Good'} · "Again" cards come back later in this session
                </div>
              </div>
            {/if}
          </div>
        </div>
      {:else if !sessionComplete}
        <div class="loading">Loading card...</div>
      {:else}
        <div class="session-complete">
          <h2>Session Complete!</h2>
          <div class="session-summary">
            <div class="summary-stat">
              <span class="stat-label">Reviews Completed:</span>
              <span class="stat-value">{sessionReviews}</span>
            </div>
            <div class="summary-stat">
              <span class="stat-label">Accuracy:</span>
              <span class="stat-value">{sessionAccuracy}</span>
            </div>
            <div class="summary-stat">
              <span class="stat-label">Time Spent:</span>
              <span class="stat-value">{formatTimeSpent((sessionEndTime || Date.now()) - sessionStartTime)}</span>
            </div>
          </div>
          {#if missedCharacters.length > 0}
            <div class="missed-kanji">
              <h3>Missed this session</h3>
              <div class="missed-list">
                {#each missedCharacters as character}
                  <a class="missed-item" href={`/kanji_view?kanji=${encodeURIComponent(character)}`} title="Open in the Kanji Database">{character}</a>
                {/each}
              </div>
            </div>
          {/if}
          <button class="continue-button" on:click={goToDashboard}>
            Back to Dashboard
          </button>
        </div>
      {/if}
    </div>
  {:else if currentView === 'stats'}
    <!-- Detailed Stats View -->
    <div class="detailed-stats">
      <h1>Detailed Statistics</h1>
      {#if studyStats}
        <!-- Overall Progress -->
        <div class="stats-section">
          <h2>Overall Progress</h2>
          <div class="progress-grid">
            <div class="progress-item">
              <div class="progress-label">Total Cards</div>
              <div class="progress-value">{studyStats.total_cards}</div>
            </div>
            <div class="progress-item">
              <div class="progress-label">Cards Due</div>
              <div class="progress-value">{studyStats.cards_due}</div>
            </div>
            <div class="progress-item">
              <div class="progress-label">Learning</div>
              <div class="progress-value">{studyStats.cards_learning}</div>
            </div>
            <div class="progress-item">
              <div class="progress-label">Mature</div>
              <div class="progress-value">{studyStats.cards_mature}</div>
            </div>
            <div class="progress-item">
              <div class="progress-label">Daily Streak</div>
              <div class="progress-value">{plural(studyStats.daily_streak, 'day')}</div>
            </div>
            <div class="progress-item">
              <div class="progress-label">Overall Accuracy</div>
              <div class="progress-value">{(studyStats.accuracy * 100).toFixed(1)}%</div>
            </div>
          </div>
        </div>

        <!-- Weekly Progress Details -->
        <div class="stats-section">
          <h2>Weekly Progress</h2>
          <div class="weekly-details">
            <div class="weekly-chart">
              <div class="chart-header">
                <span>Reviews per Day</span>
                <span>Accuracy</span>
              </div>
              {#each studyStats.reviews_this_week as reviews, i}
                <div class="weekly-row">
                  <div class="day-name">{weekdayLabel(studyStats.week_dates[i], 'long')}</div>
                  <div class="reviews-bar">
                    <div class="bar-bg">
                      <div 
                        class="bar-fill" 
                        style="width: {Math.max(2, (reviews / Math.max(...studyStats.reviews_this_week, 1)) * 100)}%"
                      ></div>
                    </div>
                    <span class="reviews-count">{reviews}</span>
                  </div>
                  <div class="accuracy-display">
                    <div class="accuracy-circle" style="--percentage: {studyStats.accuracy_this_week[i] * 100}">
                      <span>{(studyStats.accuracy_this_week[i] * 100).toFixed(0)}%</span>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>

        <!-- Study Milestones -->
        <div class="stats-section">
          <h2>Study Milestones</h2>
          <div class="milestones-grid">
            <div class="milestone-item {studyStats.total_reviews >= 100 ? 'achieved' : 'not-achieved'}">
              <div class="milestone-icon">🎯</div>
              <div class="milestone-title">First 100 Reviews</div>
              <div class="milestone-desc">{studyStats.total_reviews >= 100 ? 'Achieved!' : `${studyStats.total_reviews}/100`}</div>
            </div>
            <div class="milestone-item {studyStats.total_reviews >= 500 ? 'achieved' : 'not-achieved'}">
              <div class="milestone-icon">🏆</div>
              <div class="milestone-title">500 Reviews</div>
              <div class="milestone-desc">{studyStats.total_reviews >= 500 ? 'Achieved!' : `${studyStats.total_reviews}/500`}</div>
            </div>
            <div class="milestone-item {studyStats.total_reviews >= 1000 ? 'achieved' : 'not-achieved'}">
              <div class="milestone-icon">🌟</div>
              <div class="milestone-title">1000 Reviews</div>
              <div class="milestone-desc">{studyStats.total_reviews >= 1000 ? 'Achieved!' : `${studyStats.total_reviews}/1000`}</div>
            </div>
            <div class="milestone-item {studyStats.daily_streak >= 7 ? 'achieved' : 'not-achieved'}">
              <div class="milestone-icon">🔥</div>
              <div class="milestone-title">7 Day Streak</div>
              <div class="milestone-desc">{studyStats.daily_streak >= 7 ? 'Achieved!' : `${studyStats.daily_streak}/7 days`}</div>
            </div>
            <div class="milestone-item {studyStats.daily_streak >= 30 ? 'achieved' : 'not-achieved'}">
              <div class="milestone-icon">💪</div>
              <div class="milestone-title">30 Day Streak</div>
              <div class="milestone-desc">{studyStats.daily_streak >= 30 ? 'Achieved!' : `${studyStats.daily_streak}/30 days`}</div>
            </div>
            <div class="milestone-item {studyStats.cards_mature >= 100 ? 'achieved' : 'not-achieved'}">
              <div class="milestone-icon">🎓</div>
              <div class="milestone-title">100 Mature Cards</div>
              <div class="milestone-desc">{studyStats.cards_mature >= 100 ? 'Achieved!' : `${studyStats.cards_mature}/100`}</div>
            </div>
          </div>
        </div>

        <!-- Performance Insights -->
        <div class="stats-section">
          <h2>Performance Insights</h2>
          <div class="insights-grid">
            <div class="insight-card">
              <div class="insight-title">Study Consistency</div>
              <div class="insight-value">
                {#if studyStats.daily_streak >= 14}
                  Excellent! 🌟
                {:else if studyStats.daily_streak >= 7}
                  Good progress! 📈
                {:else if studyStats.daily_streak >= 3}
                  Keep it up! 💪
                {:else}
                  Try to study daily! 📚
                {/if}
              </div>
              <div class="insight-desc">
                {#if studyStats.daily_streak >= 14}
                  You're maintaining an excellent study streak!
                {:else if studyStats.daily_streak >= 7}
                  Great job studying consistently this week!
                {:else if studyStats.daily_streak >= 3}
                  You're building a good habit. Keep going!
                {:else}
                  Daily practice helps with retention. Try to study a little each day!
                {/if}
              </div>
            </div>
            <div class="insight-card">
              <div class="insight-title">Accuracy Level</div>
              <div class="insight-value">
                {#if studyStats.accuracy >= 0.85}
                  Excellent! 🎯
                {:else if studyStats.accuracy >= 0.75}
                  Good! 👍
                {:else if studyStats.accuracy >= 0.65}
                  Fair 📊
                {:else}
                  Needs work 📖
                {/if}
              </div>
              <div class="insight-desc">
                {#if studyStats.accuracy >= 0.85}
                  Your accuracy is outstanding! You're really mastering these kanji.
                {:else if studyStats.accuracy >= 0.75}
                  Good accuracy rate! You're on the right track.
                {:else if studyStats.accuracy >= 0.65}
                  Keep practicing! Consider reviewing cards more carefully.
                {:else}
                  Focus on understanding each kanji thoroughly before moving on.
                {/if}
              </div>
            </div>
          </div>
        </div>

        <button class="back-button" on:click={goToDashboard}>
          Back to Dashboard
        </button>
      {/if}
    </div>
  {/if}
</main>

<svelte:window on:keydown={handleKeydown} />
