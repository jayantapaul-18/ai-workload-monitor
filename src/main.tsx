import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { TrayPopupApp } from "./TrayPopupApp";

const isTrayPopup = new URLSearchParams(window.location.search).get("view") === "tray";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {isTrayPopup ? <TrayPopupApp /> : <App />}
  </React.StrictMode>,
);
