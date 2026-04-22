import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";
import styles from "./App.module.css";
import { SpaceList } from "./components/SpaceList";
import { useSpaces } from "./hooks/useSpaces";

export default function App() {
  const { spaces, loading, switchToSpace, renameSpace } = useSpaces();

  useEffect(() => {
    const win = getCurrentWindow();

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") win.hide();
    };
    document.addEventListener("keydown", onKeyDown);

    const unlistenPromise = win.onFocusChanged(({ payload: focused }) => {
      if (!focused) win.hide();
    });

    return () => {
      document.removeEventListener("keydown", onKeyDown);
      unlistenPromise.then((f) => f());
    };
  }, []);

  return (
    <div className={styles.container}>
      <div className={styles.header}>Spaces</div>
      {loading ? (
        <div className={styles.loading}>Loading…</div>
      ) : (
        <SpaceList
          spaces={spaces}
          onSwitch={switchToSpace}
          onRename={renameSpace}
        />
      )}
    </div>
  );
}
