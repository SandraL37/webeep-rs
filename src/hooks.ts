import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type LoginState = {
  state: "NotLogged" | "Logging" | "Logged" | "Error";
  token: string | null;
  error: string | null;
};

const STATE_EVENT = "login-state-changed";

export function useLoginState() {
  const [state, setState] = useState<LoginState>({
    state: "NotLogged",
    token: null,
    error: null,
  });
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    (async () => {
      const initial = await invoke<LoginState>("get_login_state");
      if (cancelled) return;
      setState(initial);
      setReady(true);

      unlisten = await listen<LoginState>(STATE_EVENT, (event) => {
        setState(event.payload);
      });
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const login = useCallback(() => invoke("login"), []);
  const logout = useCallback(() => invoke("logout"), []);

  return { state, ready, login, logout };
}

export function useLoginToken() {
  const { state } = useLoginState();
  if (state.state !== "Logged") return null;
  return state.token;
}
