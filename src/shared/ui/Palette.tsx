import { useState, type KeyboardEvent } from "react";
import styles from "./Palette.module.css";

export type PaletteOption = { id: string; label: string; detail?: string };

type PaletteProps = {
  label: string;
  placeholder: string;
  query: string;
  options: PaletteOption[];
  onQuery: (query: string) => void;
  onPick: (option: PaletteOption) => void;
  onClose: () => void;
};

export function Palette({
  label,
  placeholder,
  query,
  options,
  onQuery,
  onPick,
  onClose,
}: PaletteProps) {
  const [active, setActive] = useState(0);
  const current = Math.min(active, Math.max(options.length - 1, 0));
  const handleKey = (event: KeyboardEvent) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActive(Math.min(current + 1, options.length - 1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActive(Math.max(current - 1, 0));
    } else if (event.key === "Enter") {
      event.preventDefault();
      const option = options[current];
      if (option) {
        onPick(option);
      }
    } else if (event.key === "Escape") {
      onClose();
    }
  };
  return (
    <div className={styles.backdrop} onClick={onClose}>
      <div
        role="dialog"
        aria-label={label}
        className={styles.palette}
        onClick={(event) => event.stopPropagation()}
      >
        <input
          role="combobox"
          aria-expanded="true"
          aria-controls="palette-options"
          aria-label={label}
          autoFocus
          placeholder={placeholder}
          value={query}
          onChange={(event) => {
            setActive(0);
            onQuery(event.target.value);
          }}
          onKeyDown={handleKey}
        />
        <ul id="palette-options" role="listbox" aria-label={label}>
          {options.map((option, index) => (
            <li
              key={option.id}
              role="option"
              aria-selected={index === current}
              onMouseEnter={() => setActive(index)}
              onClick={() => onPick(option)}
            >
              <span>{option.label}</span>
              {option.detail && <small>{option.detail}</small>}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
