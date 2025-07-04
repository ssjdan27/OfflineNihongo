<script>
  import { goto } from '$app/navigation';
</script>

<main class="home-container">
  <h1>Welcome to OfflineNihongo 🎌</h1>
  <p>This app helps you look up kanji and learn Japanese offline.</p>

  <div class="button-group">
    <button on:click={() => goto('/kana_view')}>Go to Kana View</button>
    <button on:click={() => goto('/kana_game')}>Play Kana Speed Game</button>
    <button on:click={() => goto('/kanji_view')}>Browse Kanji Database</button>
    <button on:click={() => goto('/flashcards')}>Kanji Flashcards (SRS)</button>
    <button on:click={() => goto('/settings')}>Settings</button>
  </div>
</main>

<style>
  .home-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 20vh;
    text-align: center;
  }

  .button-group {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 2rem;
  }

  button {
    padding: 0.8em 1.6em;
    font-size: 1.1rem;
    border-radius: 8px;
    cursor: pointer;
    border: 2px solid #396cd8;
    background: white;
    color: #396cd8;
    transition: all 0.2s;
  }

  button:hover {
    background: #396cd8;
    color: white;
  }
</style>
