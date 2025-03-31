<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  let character = '';
  let meaning = '';
  let onyomi = '';
  let kunyomi = '';
  let jlpt = '';
  let error = '';

  async function searchKanji(event: Event) {
    event.preventDefault();
    error = '';
    meaning = onyomi = kunyomi = jlpt = '';

    try {
      const result = await invoke<{
        meaning: string;
        onyomi: string;
        kunyomi: string;
        jlpt_level: number;
      }>('get_kanji', { character });

      meaning = result.meaning;
      onyomi = result.onyomi;
      kunyomi = result.kunyomi;
      jlpt = result.jlpt_level.toString();
    } catch (e) {
      error = 'Kanji not found or error occurred.';
    }
  }
</script>

<main class="container">
  <h1>Kanji Lookup</h1>

  <form on:submit={searchKanji}>
    <input
      bind:value={character}
      placeholder="Enter a kanji (e.g. 日)"
      required
    />
    <button type="submit">Search</button>
  </form>

  {#if error}
    <p style="color: red;">{error}</p>
  {:else if meaning}
    <div class="result">
      <h2>{character}</h2>
      <p><strong>Meaning:</strong> {meaning}</p>
      <p><strong>On'yomi:</strong> {onyomi}</p>
      <p><strong>Kun'yomi:</strong> {kunyomi}</p>
      <p><strong>JLPT Level:</strong> {jlpt}</p>
    </div>
  {/if}
</main>

<style>
  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 10vh;
    text-align: center;
  }

  form {
    margin: 1em 0;
  }

  input {
    padding: 0.6em;
    font-size: 1.1em;
    border-radius: 8px;
    border: 1px solid #ccc;
    margin-right: 0.5em;
    width: 200px;
  }

  button {
    padding: 0.6em 1.2em;
    font-size: 1em;
    border-radius: 8px;
    border: 1px solid #ccc;
    cursor: pointer;
  }

  .result {
    margin-top: 2em;
    text-align: left;
    max-width: 300px;
  }

  h2 {
    font-size: 3rem;
    margin-bottom: 0.2em;
  }
</style>
