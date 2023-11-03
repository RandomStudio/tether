import reactLogo from "./assets/react.svg";
import viteLogo from "/vite.svg";
import "./App.css";
import { Tether } from "./Tether/Tether";
import { useState } from "react";

function App() {
  const [hostInput, setHostInput] = useState("localhost");
  const [host, setHost] = useState("localhost");
  return (
    <>
      <div>
        <a href="https://vitejs.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
        <input
          type="text"
          value={hostInput}
          onChange={(e) => setHostInput(e.target.value)}
        ></input>
        <button onClick={() => setHost(hostInput)}>(Re)connect</button>
        <Tether host={host} />
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  );
}

export default App;
