import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./index.css";

// Hide splash screen after React mounts
const hideSplash = () => {
  const splash = document.getElementById("splash");
  if (splash) {
    splash.classList.add("hide");
    setTimeout(() => splash.remove(), 300);
  }
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

// Hide splash after initial render
requestAnimationFrame(() => {
  requestAnimationFrame(hideSplash);
});
