<script lang="ts">
  // A square drawing pad for practicing writing a character, with an
  // optional faint guide behind the ink and a stroke counter.
  import { onMount, onDestroy } from 'svelte';

  /** The character being practiced (shown as the guide) */
  export let character: string;
  /** Stroke count of the character, if known */
  export let expectedStrokes = 0;
  /** Width and height in CSS pixels */
  export let size = 260;

  type Point = { x: number; y: number };

  let canvas: HTMLCanvasElement;
  let context: CanvasRenderingContext2D | null = null;
  let strokes: Point[][] = [];
  let drawing: Point[] | null = null;
  let showGuide = true;
  let practicedCharacter = '';

  $: inkWidth = Math.max(4, size / 26);
  $: guideFontSize = (size * 0.78) / Math.max(1, Array.from(character).length);

  // Start over when the character changes (e.g. previous/next in a list)
  $: if (character !== practicedCharacter) {
    practicedCharacter = character;
    strokes = [];
    drawing = null;
    redraw();
  }

  $: if (canvas) setupCanvas(size);

  onMount(() => {
    window.addEventListener('resize', redraw);
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') window.removeEventListener('resize', redraw);
  });

  function setupCanvas(cssSize: number) {
    const ratio = window.devicePixelRatio || 1;
    canvas.width = Math.round(cssSize * ratio);
    canvas.height = Math.round(cssSize * ratio);
    context = canvas.getContext('2d');
    context?.setTransform(ratio, 0, 0, ratio, 0, 0);
    redraw();
  }

  function drawGrid(ctx: CanvasRenderingContext2D) {
    // Dashed centre lines, like a practice square
    ctx.save();
    ctx.strokeStyle = 'rgba(0, 0, 0, 0.15)';
    ctx.lineWidth = 1;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(size / 2, 0);
    ctx.lineTo(size / 2, size);
    ctx.moveTo(0, size / 2);
    ctx.lineTo(size, size / 2);
    ctx.stroke();
    ctx.restore();
  }

  function drawStroke(ctx: CanvasRenderingContext2D, points: Point[]) {
    if (points.length === 0) return;
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    if (points.length === 1) {
      // A tap still leaves a dot
      ctx.lineTo(points[0].x + 0.01, points[0].y + 0.01);
    }
    for (const point of points.slice(1)) {
      ctx.lineTo(point.x, point.y);
    }
    ctx.stroke();
  }

  function setInk(ctx: CanvasRenderingContext2D) {
    ctx.strokeStyle = '#222';
    ctx.lineWidth = inkWidth;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
  }

  function redraw() {
    const ctx = context;
    if (!ctx) return;
    ctx.clearRect(0, 0, size, size);
    drawGrid(ctx);
    setInk(ctx);
    for (const stroke of strokes) drawStroke(ctx, stroke);
  }

  function pointFrom(event: PointerEvent): Point {
    const rect = canvas.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / rect.width) * size,
      y: ((event.clientY - rect.top) / rect.height) * size
    };
  }

  function startStroke(event: PointerEvent) {
    if (event.button !== 0) return;
    event.preventDefault();
    canvas.setPointerCapture(event.pointerId);
    drawing = [pointFrom(event)];
    strokes = [...strokes, drawing];
    if (context) {
      setInk(context);
      drawStroke(context, drawing);
    }
  }

  function continueStroke(event: PointerEvent) {
    if (!drawing || !context) return;
    const previous = drawing[drawing.length - 1];
    const point = pointFrom(event);
    drawing.push(point);
    setInk(context);
    context.beginPath();
    context.moveTo(previous.x, previous.y);
    context.lineTo(point.x, point.y);
    context.stroke();
  }

  function endStroke() {
    drawing = null;
  }

  function undo() {
    strokes = strokes.slice(0, -1);
    redraw();
  }

  function clear() {
    strokes = [];
    redraw();
  }
</script>

<div class="writing-pad">
  <div class="pad" style="width: {size}px; height: {size}px;">
    {#if showGuide}
      <div class="guide" style="font-size: {guideFontSize}px;" aria-hidden="true">{character}</div>
    {/if}
    <canvas
      bind:this={canvas}
      style="width: {size}px; height: {size}px;"
      aria-label={`Writing practice for ${character}`}
      on:pointerdown={startStroke}
      on:pointermove={continueStroke}
      on:pointerup={endStroke}
      on:pointercancel={endStroke}
      on:lostpointercapture={endStroke}
    ></canvas>
  </div>

  <div class="pad-controls">
    <span class="stroke-count">
      Strokes: {strokes.length}{expectedStrokes > 0 ? ` / ${expectedStrokes}` : ''}
    </span>
    <button type="button" on:click={undo} disabled={strokes.length === 0}>Undo</button>
    <button type="button" on:click={clear} disabled={strokes.length === 0}>Clear</button>
    <label class="guide-toggle">
      <input type="checkbox" bind:checked={showGuide} />
      Guide
    </label>
  </div>
</div>

<style>
  .writing-pad {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
    padding: 0.75rem;
  }

  .pad {
    position: relative;
    border-radius: 6px;
    border: 1px solid var(--border-color, #e0e0e0);
    background-color: white;
    overflow: hidden;
  }

  .guide {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--font-jp-display);
    line-height: 1;
    color: rgba(0, 0, 0, 0.12);
    user-select: none;
    pointer-events: none;
  }

  canvas {
    position: relative;
    display: block;
    touch-action: none;
    cursor: crosshair;
  }

  .pad-controls {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    justify-content: center;
    font-size: 0.85rem;
  }

  .pad-controls button {
    padding: 0.3rem 0.7rem;
    font-size: 0.85rem;
    border-radius: 4px;
    border: 1px solid var(--button-border, #ccc);
    background-color: var(--button-bg, #f3f3f3);
    color: var(--text-color, #000);
    cursor: pointer;
  }

  .pad-controls button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .stroke-count {
    color: var(--text-muted, #666);
    font-variant-numeric: tabular-nums;
  }

  .guide-toggle {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--text-muted, #666);
    cursor: pointer;
  }
</style>
