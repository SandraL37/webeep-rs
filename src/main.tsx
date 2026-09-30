import React from "react";
import ReactDOM from "react-dom/client";
import { Theme, ThemePanel } from "@radix-ui/themes";
import App from "./App";

import "@radix-ui/themes/styles.css";
import "./main.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Theme appearance="dark">
      <App />
      <ThemePanel defaultOpen={false} />
    </Theme>
  </React.StrictMode>,
);
