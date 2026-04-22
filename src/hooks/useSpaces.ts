import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import type { NameMap, Space, SpaceInfo } from "../types";

export function useSpaces() {
  const [spaces, setSpaces] = useState<Space[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const [rawSpaces, nameMap] = await Promise.all([
        invoke<SpaceInfo[]>("get_spaces"),
        invoke<NameMap>("get_space_names"),
      ]);
      setSpaces(
        rawSpaces.map((s) => ({
          ...s,
          displayName: nameMap[s.uuid] ?? `Desktop ${s.index}`,
        }))
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const win = getCurrentWindow();
    // Re-fetch spaces each time the popup opens so active space is current
    const unlistenPromise = win.onFocusChanged(({ payload: focused }) => {
      if (focused) refresh();
    });
    return () => {
      unlistenPromise.then((f) => f());
    };
  }, [refresh]);

  const switchToSpace = useCallback(
    async (spaceId: number, displayId: string) => {
      await invoke("cmd_switch_to_space", { spaceId, displayId });
      await getCurrentWindow().hide();
    },
    []
  );

  const renameSpace = useCallback(
    async (uuid: string, name: string) => {
      await invoke("set_space_name", { spaceUuid: uuid, name });
      await refresh();
    },
    [refresh]
  );

  return { spaces, loading, refresh, switchToSpace, renameSpace };
}
