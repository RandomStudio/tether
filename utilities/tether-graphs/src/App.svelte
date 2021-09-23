<script>
  export let files;

  $: if (files) {
    console.log("file", files);
  }

  function drawGraph(file) {
    console.log("read file", file);
    const reader = new FileReader();
    reader.onload = (e) => {
      console.log("data:", e.target.result);
    };
    reader.readAsText(file);
  }
</script>

<main>
  <label for="avatar">Upload a picture:</label>
  <input
    accept="application/json"
    bind:files
    id="jsonFile"
    name="jsonFile"
    type="file"
  />

  {#if files}
    <h2>Selected files:</h2>
    {#each Array.from(files) as file}
      <p>{file.name} ({file.size} bytes)</p>
      <button on:click={drawGraph(file)}>Draw Graph</button>
    {/each}
  {/if}
</main>

<style>
  main {
    text-align: center;
    padding: 1em;
    max-width: 240px;
    margin: 0 auto;
  }

  h1 {
    color: #ff3e00;
    text-transform: uppercase;
    font-size: 4em;
    font-weight: 100;
  }

  @media (min-width: 640px) {
    main {
      max-width: none;
    }
  }
</style>
