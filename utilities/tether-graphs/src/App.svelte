<script>
  import { Chart, registerables } from "chart.js";
  Chart.register(...registerables);

  let files;
  let canvas;

  $: if (files) {
    console.log("file", files);
  }

  const findTopics = (jsonArray) => {
    let topics = [];
    jsonArray.forEach((entry) => {
      if (!topics.includes(entry.topic)) {
        topics.push(entry.topic);
      }
    });
    return topics;
  };

  const findLabels = (filteredArray) => {
    let labels = [];
    filteredArray.forEach((entry) => {
      if (!labels.includes(entry.timestamp)) {
        labels.push(entry.timestamp);
      }
    });
    return labels;
  };

  function drawGraph(jsonString, valueKey) {
    const jsonArray = JSON.parse(jsonString).filter(
      (e) => e[valueKey] !== undefined
    );

    const topics = findTopics(jsonArray);
    console.log({ jsonArray, topics });

    const data = {
      datasets: topics.map((topic) => ({
        label: topic,
        data: jsonArray
          .filter((e) => e.topic === topic)
          .map((e) => ({
            x: e.timestamp,
            y: e[valueKey],
          })),
      })),
      // [
      //   {
      //     label: "incoming",
      //     data: jsonData.map((e) => e.incomingValue),
      //   },
      //   {
      //     label: "filtered",
      //     data: jsonData.map((e) => e.filteredValue),
      //     borderColor: "rgb(75, 192, 192)",
      //   },
      // ],
      labels: findLabels(jsonArray),
    };

    const config = {
      type: "line",
      data,
      options: {
        scales: {
          y: {
            title: {
              display: true,
              text: valueKey,
            },
          },
          x: {
            title: {
              display: true,
              text: "timestamp",
            },
          },
        },
      },
    };

    console.log({ data });

    const ctx = canvas.getContext("2d");
    const c = new Chart(ctx, config);
  }

  function uploadFile(file) {
    console.log("read file", file);
    const reader = new FileReader();
    reader.onload = (e) => {
      console.log("data:", e.target.result);
      drawGraph(e.target.result, "position");
    };
    reader.readAsText(file);
  }
</script>

<main>
  <h1>Tether Graph Utility</h1>

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
