import { useRef, useState } from "react";
import type { Space } from "../types";
import styles from "./SpaceItem.module.css";

interface Props {
  space: Space;
  onSwitch: () => void;
  onRename: (name: string) => Promise<void>;
}

export function SpaceItem({ space, onSwitch, onRename }: Props) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(space.displayName);
  const inputRef = useRef<HTMLInputElement>(null);

  const startEdit = (e: React.MouseEvent) => {
    e.stopPropagation();
    setDraft(space.displayName);
    setEditing(true);
    requestAnimationFrame(() => inputRef.current?.select());
  };

  const commitEdit = async () => {
    setEditing(false);
    const trimmed = draft.trim();
    if (trimmed !== space.displayName) {
      await onRename(trimmed);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      commitEdit();
    } else if (e.key === "Escape") {
      e.stopPropagation();
      setEditing(false);
      setDraft(space.displayName);
    }
  };

  return (
    <li
      className={`${styles.item} ${space.active ? styles.active : ""}`}
      onClick={editing ? undefined : onSwitch}
      role="option"
      aria-selected={space.active}
    >
      <span
        className={`${styles.dot} ${space.active ? styles.dotActive : ""}`}
        aria-hidden="true"
      />

      {editing ? (
        <input
          ref={inputRef}
          className={styles.input}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commitEdit}
          onKeyDown={handleKeyDown}
          maxLength={40}
          autoFocus
          onClick={(e) => e.stopPropagation()}
        />
      ) : (
        <span
          className={styles.name}
          onDoubleClick={startEdit}
          title="Double-click to rename"
        >
          {space.displayName}
        </span>
      )}

      <span className={styles.badge}>Desktop {space.index}</span>
    </li>
  );
}
