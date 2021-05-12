import React from "react";
import "./App.css";
import Connection from "./Connection/Connection";

function App() {
  return (
    <div className="App">
      <header className="App-header">Tether2 Explorer</header>
      <Connection path="/ws" address="ws://localhost" port={15675} />
    </div>
  );
}

export default App;
