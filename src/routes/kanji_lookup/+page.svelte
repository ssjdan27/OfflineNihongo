<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { afterUpdate } from 'svelte';

  let submitted_kanji = "";
  let character = "";
  let stroke_count = "";
  let grade = "";
  let jlpt_level = "";
  let frequency = "";
  let onyomi = "";
  let kunyomi = "";
  let meanings: string[] = [];
  let nanori: string[] = [];
  let kanji_svg: string = "";
  let error = "";

  async function searchKanji(event: Event) {
    event.preventDefault();
    error = "";
    meanings = [];
    onyomi = kunyomi = jlpt_level = "";
    kanji_svg = "";

    try {
      const result = await invoke<{
        character: string;
        stroke_count: number;
        grade: number;
        jlpt_level: number;
        frequency: number;
        onyomi: string;
        kunyomi: string;
        meanings: string[];
        nanori: string[];
      }>("get_kanji", { character: submitted_kanji });

      const svg_result = await invoke<string>("get_kanji_svg", {
        character: submitted_kanji,
      });
      kanji_svg = svg_result;

      character = result.character;
      stroke_count = result.stroke_count.toString();
      grade = result.grade.toString();
      jlpt_level = result.jlpt_level.toString();
      frequency = result.frequency.toString();
      onyomi = result.onyomi;
      kunyomi = result.kunyomi;
      meanings = result.meanings;
      nanori = result.nanori;
    } catch (e) {
      console.log("Error fetching kanji data:", e);
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function goToHome() {
    goto("/");
  }

  export function insertSvg(node: HTMLElement) {
  if (kanji_svg) {
    try {
      // Create a temporary container to parse the SVG string properly
      const parser = new DOMParser();
      const doc = parser.parseFromString(kanji_svg, "image/svg+xml");
      
      // Extract just the SVG element
      const svgElement = doc.querySelector("svg");
      
      if (svgElement) {
        // Clear the node and append the SVG element
        node.innerHTML = "";
        node.appendChild(svgElement.cloneNode(true));
      } else {
        console.error("No SVG element found in the string");
      }
    } catch (e) {
      console.error("Error parsing SVG:", e);
      
      // Fallback to regex cleaning if parsing fails
      const cleaned = kanji_svg
        .replace(/<\?xml[^>]*\?>/g, '')          // Remove XML declaration
        .replace(/<!DOCTYPE[^>]*>/g, '')         // Remove DOCTYPE
        .replace(/<!ENTITY[^>]*>/g, '')          // Remove ENTITY declarations
        .replace(/<!\[CDATA\[.*?\]\]>/gs, '')    // Remove CDATA sections
        .replace(/\]\s*>$/g, '')                 // Remove trailing ]>
        .replace(/\s*\]\s*>$/g, '');             // Remove trailing ]> with whitespace
        
      const svgStart = cleaned.indexOf("<svg");
      const svgEnd = cleaned.lastIndexOf("</svg>") + 6; // +6 for "</svg>"
      
      if (svgStart >= 0 && svgEnd > svgStart) {
        node.innerHTML = cleaned.substring(svgStart, svgEnd);
      } else {
        node.innerHTML = cleaned;
      }
    }
  }
}
</script>

<main class="container">
  <h1>Kanji Lookup</h1>

  <form on:submit={searchKanji}>
    <input
      bind:value={submitted_kanji}
      placeholder="Enter a kanji (e.g. 日)"
      required
    />
    <button type="submit">Search</button>
  </form>

  <button class="home-button" on:click={goToHome}> Back to Home </button>

  {#if error}
    <p style="color: red;">{error}</p>
  {:else if meanings.length > 0}
    <div class="result">
      <h2>{character}</h2>
      <p><strong>Stroke Count: {stroke_count}</strong></p>
      <p><strong>Grade: {grade}</strong></p>
      <p><strong>JLPT Level: {jlpt_level}</strong></p>
      <p><strong>Frequency: {frequency}</strong></p>
      <p><strong>Onyomi: {onyomi}</strong></p>
      <p><strong>Kunyomi: {kunyomi}</strong></p>
      <p><strong>Meanings:</strong></p>
      <ul>
        {#each meanings as meaning}
          <li>{meaning}</li>
        {/each}
      </ul>
      <p><strong>Nanori:</strong></p>
      <ul>
        {#each nanori as name}
          <li>{name}</li>
        {/each}
      </ul>
      {#if kanji_svg}
        <div class="svg-container" use:insertSvg></div>
      {/if}
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

  .home-button {
    margin-top: 1rem;
    padding: 0.6em 1.2em;
    font-size: 1em;
    border-radius: 8px;
    cursor: pointer;
    background-color: #f2f2f2;
    border: 1px solid #ccc;
  }
</style>
