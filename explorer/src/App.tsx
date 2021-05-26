import { useState } from "react";
import "./App.css";
import Connection from "./Connection/Connection";

function App() {
  const [host, setHost] = useState("localhost");
  const [port, setPort] = useState(15675);
  const [ready, setReady] = useState(false);

  return (
    <div className="App">
      <header className="App-header">Tether2 Explorer</header>
      {ready ? (
        <Connection path="/ws" host={host} port={port} />
      ) : (
        <div>
          <h2>Enter host details</h2>
          <div>
            <input
              type="text"
              value={host}
              onChange={(e) => {
                setHost(e.target.value);
              }}
            ></input>
            <input
              type="number"
              value={port}
              onChange={(e) => {
                setPort(parseInt(e.target.value));
              }}
            ></input>
          </div>
          <button
            onClick={() => {
              setReady(true);
            }}
          >
            Connect to {`${host}:${port}...`}
          </button>
        </div>
      )}
    </div>
  );
}

export default App;
