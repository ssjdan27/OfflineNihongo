<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';

  interface StudyStats {
    total_cards: number;
    cards_due: number;
    cards_new: number;
    daily_streak: number;
  }

  let stats: StudyStats | null = null;

  const sections = [
    {
      path: '/kana_view',
      icon: 'あ',
      title: 'Kana Chart',
      description: 'Hiragana and katakana with stroke order'
    },
    {
      path: '/kana_game',
      icon: '⏱',
      title: 'Kana Speed Game',
      description: 'Type the romaji for each kana as fast as you can'
    },
    {
      path: '/kanji_view',
      icon: '字',
      title: 'Kanji Database',
      description: '13,000 kanji: search by meaning, reading or romaji'
    },
    {
      path: '/flashcards',
      icon: '札',
      title: 'Kanji Flashcards',
      description: 'Spaced repetition reviews of meanings and readings'
    },
    {
      path: '/settings',
      icon: '⚙',
      title: 'Settings',
      description: 'Theme, study session size and backups'
    }
  ];

  onMount(async () => {
    try {
      stats = await invoke('get_study_stats');
    } catch (err) {
      // The summary is optional; the navigation still works without it
      console.error('Failed to load study stats:', err);
    }
  });

  function plural(count: number, word: string): string {
    return `${count} ${count === 1 ? word : word + 's'}`;
  }

  $: hasWork = stats !== null && stats.cards_due + stats.cards_new > 0;
</script>

<main class="home-container">
  <h1>OfflineNihongo <span class="flag" aria-hidden="true">🎌</span></h1>
  <p class="tagline">Learn kana and kanji, fully offline.</p>

  {#if stats}
    <section class="today" aria-label="Today">
      {#if stats.total_cards === 0}
        <div class="today-text">
          <strong>Start learning kanji</strong>
          <span>Add your first flashcards and review them a little each day.</span>
        </div>
        <button class="today-button" on:click={() => goto('/flashcards')}>Get started</button>
      {:else}
        <div class="today-text">
          {#if hasWork}
            <strong>
              {[
                stats.cards_due > 0 ? `${plural(stats.cards_due, 'review')} due` : '',
                stats.cards_new > 0 ? plural(stats.cards_new, 'new card') : ''
              ]
                .filter(Boolean)
                .join(' · ')}
            </strong>
          {:else}
            <strong>All caught up for today</strong>
          {/if}
          <span>
            {#if stats.daily_streak > 0}
              🔥 {stats.daily_streak}-day streak
            {:else}
              Review today to start a streak
            {/if}
          </span>
        </div>
        {#if hasWork}
          <button class="today-button" on:click={() => goto('/flashcards?start=mixed')}>Study now</button>
        {/if}
      {/if}
    </section>
  {/if}

  <nav class="sections">
    {#each sections as section}
      <button class="section-tile" on:click={() => goto(section.path)}>
        <span class="section-icon" aria-hidden="true">{section.icon}</span>
        <span class="section-text">
          <span class="section-title">{section.title}</span>
          <span class="section-description">{section.description}</span>
        </span>
      </button>
    {/each}
  </nav>
</main>

<style>
  .home-container {
    max-width: 640px;
    margin: 0 auto;
    padding: 12vh 1.5rem 3rem;
    text-align: center;
  }

  h1 {
    margin: 0;
    font-size: 2.2rem;
  }

  .tagline {
    margin: 0.5rem 0 2rem;
    color: var(--text-muted, #666);
  }

  .today {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.5rem;
    padding: 1rem 1.25rem;
    text-align: left;
    border-radius: 12px;
    border: 1px solid var(--accent-color, #396cd8);
    background-color: var(--section-bg, #fff);
  }

  .today-text {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .today-text span {
    font-size: 0.9rem;
    color: var(--text-muted, #666);
  }

  .today-button {
    flex-shrink: 0;
    padding: 0.6rem 1.2rem;
    font-size: 1rem;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    background-color: var(--accent-color, #396cd8);
    color: white;
    cursor: pointer;
  }

  .today-button:hover {
    background-color: var(--accent-color, #396cd8);
    filter: brightness(1.1);
  }

  .sections {
    display: grid;
    gap: 0.75rem;
  }

  .section-tile {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.9rem 1.1rem;
    text-align: left;
    border-radius: 12px;
    border: 1px solid var(--border-color, #e0e0e0);
    background-color: var(--section-bg, #fff);
    color: var(--text-color, #000);
    cursor: pointer;
    transition: border-color 0.2s, transform 0.2s;
  }

  .section-tile:hover {
    border-color: var(--accent-color, #396cd8);
    background-color: var(--section-bg, #fff);
    transform: translateY(-1px);
  }

  .section-icon {
    width: 2.5rem;
    height: 2.5rem;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 10px;
    font-family: var(--font-jp-display);
    font-size: 1.5rem;
    color: var(--accent-color, #396cd8);
    background-color: var(--item-border, #f0f0f0);
  }

  .section-text {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .section-title {
    font-weight: 600;
    font-size: 1.05rem;
  }

  .section-description {
    font-size: 0.875rem;
    color: var(--text-muted, #666);
  }
</style>
