<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import './flashcards.css';

  interface KanjiCard {
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

  interface SrsCard {
    id: string;
    character: string;
    level: number;
    interval: number;
    ease_factor: number;
    next_review: string; // ISO date string
    total_reviews: number;
    correct_reviews: number;
    created_at: string;
    last_reviewed: string | null;
    streak: number;
  }

  interface ReviewSession {
    cards_due: SrsCard[];
    cards_new: SrsCard[];
    session_stats: {
      total_reviews: number;
      correct_answers: number;
      session_time: number;
    };
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
    reviews_this_week: number[];
    accuracy_this_week: number[];
  }

  // App state
  let loading = true;
  let error = '';
  let currentView = 'dashboard'; // dashboard, study, stats
  let studyStats: StudyStats | null = null;
  let reviewSession: ReviewSession | null = null;
  let currentCard: SrsCard | null = null;
  let currentKanjiData: KanjiCard | null = null;
  let showAnswer = false;
  let sessionStartTime = 0;
  let reviewType = 'meaning'; // 'meaning' or 'reading'

  onMount(async () => {
    await loadStudyStats();
    loading = false;
  });

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
      reviewSession = await invoke('start_review_session', { sessionType });
      if (reviewSession && (reviewSession.cards_due.length > 0 || reviewSession.cards_new.length > 0)) {
        currentView = 'study';
        sessionStartTime = Date.now();
        await nextCard();
      } else {
        error = 'No cards available for review';
      }
      loading = false;
    } catch (err) {
      error = `Failed to start review session: ${err}`;
      console.error('Error starting review session:', err);
      loading = false;
    }
  }

  async function nextCard() {
    if (!reviewSession) return;
    
    const allCards = [...reviewSession.cards_due, ...reviewSession.cards_new];
    if (allCards.length === 0) {
      // Session complete
      currentView = 'dashboard';
      await loadStudyStats();
      return;
    }

    // Get next card (prioritize due cards)
    currentCard = reviewSession.cards_due.length > 0 
      ? reviewSession.cards_due[0] 
      : reviewSession.cards_new[0];
    
    // Load kanji data for the card
    try {
      currentKanjiData = await invoke('get_kanji', { character: currentCard.character });
      // Randomly choose review type (60% meaning, 40% reading)
      reviewType = Math.random() < 0.6 ? 'meaning' : 'reading';
      showAnswer = false;
    } catch (err) {
      console.error('Error loading kanji data:', err);
    }
  }

  async function submitAnswer(grade: number) {
    if (!currentCard || !reviewSession) return;

    try {
      // Submit the answer and get updated card
      const updatedCard = await invoke('submit_card_review', {
        cardId: currentCard.id,
        grade: grade,
        reviewType: reviewType
      });

      // Update session stats
      reviewSession.session_stats.total_reviews += 1;
      if (grade >= 3) {
        reviewSession.session_stats.correct_answers += 1;
      }

      // Remove the current card from the session
      if (reviewSession.cards_due.includes(currentCard)) {
        reviewSession.cards_due = reviewSession.cards_due.filter(c => c.id !== currentCard!.id);
      } else {
        reviewSession.cards_new = reviewSession.cards_new.filter(c => c.id !== currentCard!.id);
      }

      // Move to next card
      await nextCard();
    } catch (err) {
      error = `Failed to submit answer: ${err}`;
      console.error('Error submitting answer:', err);
    }
  }

  function showAnswerCard() {
    showAnswer = true;
  }

  async function addNewCards(count: number) {
    try {
      await invoke('add_new_cards', { count });
      await loadStudyStats();
    } catch (err) {
      error = `Failed to add new cards: ${err}`;
      console.error('Error adding new cards:', err);
    }
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
</script>

<div class="nav-bar">
  <button class="home-button" on:click={goToHome}>Back to Home</button>
  {#if currentView !== 'dashboard'}
    <button class="back-button" on:click={() => currentView = 'dashboard'}>Dashboard</button>
  {/if}
</div>

<main class="flashcards-container">
  {#if loading}
    <div class="loading">Loading...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if currentView === 'dashboard'}
    <!-- Dashboard View -->
    <div class="dashboard">
      <h1>Kanji Flashcards</h1>
      
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
        </div>

        <!-- Quick Stats -->
        <div class="quick-stats">
          <div class="quick-stat">
            <span class="label">Daily Streak:</span>
            <span class="value">{studyStats.daily_streak} days</span>
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
                        style="height: {Math.max(4, (reviews / Math.max(...studyStats.reviews_this_week)) * 100)}px"
                        title="{reviews} reviews"
                      ></div>
                    </div>
                    <div class="day-label">
                      {['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'][i]}
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
      {#if currentCard && currentKanjiData}
        <div class="session-header">
          <div class="session-progress">
            <div class="progress-info">
              Remaining: {(reviewSession?.cards_due.length || 0) + (reviewSession?.cards_new.length || 0)} cards
            </div>
            <div class="session-stats">
              Reviews: {reviewSession?.session_stats.total_reviews || 0} | 
              Accuracy: {(reviewSession?.session_stats.total_reviews || 0) > 0 ? 
                (((reviewSession?.session_stats.correct_answers || 0) / (reviewSession?.session_stats.total_reviews || 1)) * 100).toFixed(1) + '%' : 
                'N/A'}
            </div>
          </div>
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
              <button class="show-answer-button" on:click={showAnswerCard}>
                Show Answer
              </button>
            {:else}
              <div class="answer-section">
                {#if reviewType === 'meaning'}
                  <div class="answer">
                    <h4>Meanings:</h4>
                    <div class="meanings-list">
                      {#each currentKanjiData.meanings as meaning}
                        <span class="meaning-tag">{meaning}</span>
                      {/each}
                    </div>
                  </div>
                {:else}
                  <div class="answer">
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
                  </div>
                {/if}

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
                  <button class="grade-button fail" on:click={() => submitAnswer(1)}>
                    Again<br><span class="grade-desc">Complete blackout</span>
                  </button>
                  <button class="grade-button hard" on:click={() => submitAnswer(2)}>
                    Hard<br><span class="grade-desc">Incorrect but remembered</span>
                  </button>
                  <button class="grade-button good" on:click={() => submitAnswer(3)}>
                    Good<br><span class="grade-desc">Correct with effort</span>
                  </button>
                  <button class="grade-button easy" on:click={() => submitAnswer(4)}>
                    Easy<br><span class="grade-desc">Perfect recall</span>
                  </button>
                </div>
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <div class="session-complete">
          <h2>Session Complete!</h2>
          <div class="session-summary">
            <div class="summary-stat">
              <span class="stat-label">Reviews Completed:</span>
              <span class="stat-value">{reviewSession?.session_stats.total_reviews || 0}</span>
            </div>
            <div class="summary-stat">
              <span class="stat-label">Accuracy:</span>
              <span class="stat-value">
                {(reviewSession?.session_stats.total_reviews || 0) > 0 ? 
                  (((reviewSession?.session_stats.correct_answers || 0) / (reviewSession?.session_stats.total_reviews || 1)) * 100).toFixed(1) + '%' : 
                  'N/A'}
              </span>
            </div>
            <div class="summary-stat">
              <span class="stat-label">Time Spent:</span>
              <span class="stat-value">{formatTimeSpent(Date.now() - sessionStartTime)}</span>
            </div>
          </div>
          <button class="continue-button" on:click={() => currentView = 'dashboard'}>
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
              <div class="progress-value">{studyStats.daily_streak} days</div>
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
                  <div class="day-name">{['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'][i]}</div>
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

        <button class="back-button" on:click={() => currentView = 'dashboard'}>
          Back to Dashboard
        </button>
      {/if}
    </div>
  {/if}
</main>
