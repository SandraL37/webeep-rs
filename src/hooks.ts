import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands, LoginState } from "./bindings";

const STATE_EVENT = "login-state-changed";

export function useLoginState() {
  const [state, setState] = useState<LoginState>("NotLogged");

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    (async () => {
      const initial = await commands.getLoginState();
      if (cancelled || initial.status === "error") return;
      setState(initial.data);

      unlisten = await listen<LoginState>(STATE_EVENT, (event) => {
        setState(event.payload);
      });
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return { state };
}
