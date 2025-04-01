<script>
  import { goto } from '$app/navigation';

  function goToLookup() {
    goto('/kanji_lookup');
  }
</script>

<main class="home-container">
  <h1>Welcome to OfflineNihongo 🎌</h1>
  <p>This app helps you look up kanji and learn Japanese offline.</p>

  <button on:click={goToLookup}>Go to Kanji Lookup</button>
</main>

<style>
  .home-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 20vh;
    text-align: center;
  }

  button {
    margin-top: 2rem;
    padding: 0.8em 1.6em;
    font-size: 1.1rem;
    border-radius: 8px;
    cursor: pointer;
  }
</style>
