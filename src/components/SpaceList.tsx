import type { Space } from "../types";
import { SpaceItem } from "./SpaceItem";
import styles from "./SpaceList.module.css";

interface Props {
  spaces: Space[];
  onSwitch: (id: number, displayId: string) => void;
  onRename: (uuid: string, name: string) => Promise<void>;
}

export function SpaceList({ spaces, onSwitch, onRename }: Props) {
  if (spaces.length === 0) {
    return <p className={styles.empty}>No Spaces found.</p>;
  }

  return (
    <ul className={styles.list} role="listbox">
      {spaces.map((space) => (
        <SpaceItem
          key={space.uuid}
          space={space}
          onSwitch={() => onSwitch(space.id, space.display_id)}
          onRename={(name) => onRename(space.uuid, name)}
        />
      ))}
    </ul>
  );
}
