<script>
  import { Chart, registerables } from "chart.js";
  Chart.register(...registerables);

  let files;
  let canvas;

  $: if (files) {
    console.log("file", files);
  }

  function drawGraph(jsonString) {
    const json = JSON.parse(jsonString);

    const labels = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul"];
    const data = {
      labels: labels,
      datasets: [
        {
          label: "My First Dataset",
          data: [65, 59, 80, 81, 56, 55, 40],
          fill: false,
          borderColor: "rgb(75, 192, 192)",
          tension: 0.1,
        },
      ],
    };

    const config = {
      type: "line",
      data: data,
    };

    const ctx = canvas.getContext("2d");
    const c = new Chart(ctx, config);
  }

  function uploadFile(file) {
    console.log("read file", file);
    const reader = new FileReader();
    reader.onload = (e) => {
      console.log("data:", e.target.result);
      drawGraph(e.target.result);
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
      <button on:click={uploadFile(file)}>Draw Graph</button>
    {/each}
  {/if}

  <div>
    <canvas bind:this={canvas} width={400} height={400} />
  </div>
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
