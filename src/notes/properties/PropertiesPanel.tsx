import { useId, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { PropertySummaryView } from "../../shared/bindings/PropertySummaryView";
import type { PropertyValueView } from "../../shared/bindings/PropertyValueView";
import type { PropertyView } from "../../shared/bindings/PropertyView";
import styles from "./PropertiesPanel.module.css";
import {
  emptyValue,
  isListKind,
  itemsOf,
  kindOf,
  textOf,
  textValue,
  wikilinkTarget,
  type PropertyKind,
} from "./propertyKind";

type PropertiesPanelProps = {
  properties: PropertyView[];
  catalog: PropertySummaryView[];
  editable: boolean;
  onChange: (properties: PropertyView[]) => void;
  onFollow: (target: string) => void;
};

export function PropertiesPanel({
  properties,
  catalog,
  editable,
  onChange,
  onFollow,
}: PropertiesPanelProps) {
  const { t } = useTranslation();
  const [adding, setAdding] = useState(false);
  const namesId = useId();

  if (!editable && properties.length === 0) {
    return null;
  }

  const replace = (index: number, property: PropertyView) =>
    onChange(
      properties.map((current, at) => (at === index ? property : current)),
    );
  const remove = (index: number) =>
    onChange(properties.filter((_, at) => at !== index));
  const add = (name: string) => {
    const key = name.trim();
    setAdding(false);
    if (key === "" || properties.some((property) => property.key === key)) {
      return;
    }
    const known = catalog.find((summary) => summary.name === key);
    onChange([
      ...properties,
      { key, value: emptyValue(known?.kind ?? "text") },
    ]);
  };

  return (
    <section className={styles.panel} aria-label={t("properties.title")}>
      <dl className={styles.list}>
        {properties.map((property, index) => (
          <PropertyRow
            key={`${index}-${property.key}`}
            property={property}
            kind={kindOf(property.key, property.value, catalog)}
            suggestions={
              catalog.find((summary) => summary.name === property.key)
                ?.values ?? []
            }
            editable={editable}
            onRename={(key) => {
              const name = key.trim();
              if (
                name !== "" &&
                !properties.some((other) => other.key === name)
              ) {
                replace(index, { ...property, key: name });
              }
            }}
            onValue={(value) => replace(index, { ...property, value })}
            onRemove={() => remove(index)}
            onFollow={onFollow}
          />
        ))}
      </dl>
      {editable &&
        (adding ? (
          <>
            <input
              className={styles.newName}
              autoFocus
              list={namesId}
              aria-label={t("properties.name")}
              placeholder={t("properties.name")}
              onBlur={(event) => add(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  add(event.currentTarget.value);
                } else if (event.key === "Escape") {
                  setAdding(false);
                }
              }}
            />
            <datalist id={namesId}>
              {catalog.map((summary) => (
                <option key={summary.name} value={summary.name} />
              ))}
            </datalist>
          </>
        ) : (
          <button
            type="button"
            className={styles.add}
            onClick={() => setAdding(true)}
          >
            ＋ {t("properties.add")}
          </button>
        ))}
    </section>
  );
}

type PropertyRowProps = {
  property: PropertyView;
  kind: PropertyKind;
  suggestions: string[];
  editable: boolean;
  onRename: (key: string) => void;
  onValue: (value: PropertyValueView) => void;
  onRemove: () => void;
  onFollow: (target: string) => void;
};

function PropertyRow({
  property,
  kind,
  suggestions,
  editable,
  onRename,
  onValue,
  onRemove,
  onFollow,
}: PropertyRowProps) {
  const { t } = useTranslation();
  return (
    <div className={styles.row} data-kind={kind}>
      <dt>
        <span className={styles.icon} aria-hidden="true">
          {ICONS[kind]}
        </span>
        {editable ? (
          <input
            className={styles.key}
            defaultValue={property.key}
            aria-label={t("properties.name")}
            onBlur={(event) => onRename(event.currentTarget.value)}
            onKeyDown={commitOnEnter}
          />
        ) : (
          property.key
        )}
      </dt>
      <dd>
        {editable ? (
          <ValueEditor
            label={property.key}
            kind={kind}
            value={property.value}
            suggestions={suggestions}
            onValue={onValue}
          />
        ) : (
          <ValueDisplay
            kind={kind}
            value={property.value}
            onFollow={onFollow}
          />
        )}
        {editable && (
          <button
            type="button"
            className={styles.remove}
            aria-label={t("properties.remove", { name: property.key })}
            onClick={onRemove}
          >
            ×
          </button>
        )}
      </dd>
    </div>
  );
}

function ValueDisplay({
  kind,
  value,
  onFollow,
}: {
  kind: PropertyKind;
  value: PropertyValueView;
  onFollow: (target: string) => void;
}) {
  const { i18n } = useTranslation();
  if (kind === "checkbox") {
    return (
      <input
        type="checkbox"
        checked={value.type === "boolean" && value.value}
        readOnly
        disabled
      />
    );
  }
  if (isListKind(kind)) {
    return (
      <span className={styles.chips}>
        {itemsOf(value).map((item, index) => (
          <span key={index} className={styles.chip}>
            <LinkOrText text={item} onFollow={onFollow} />
          </span>
        ))}
      </span>
    );
  }
  const text = textOf(value);
  if ((kind === "date" || kind === "dateTime") && text !== "") {
    const date = new Date(kind === "date" ? `${text}T00:00:00` : text);
    if (!Number.isNaN(date.getTime())) {
      return (
        <>
          {new Intl.DateTimeFormat(i18n.language, {
            dateStyle: "medium",
            timeStyle: kind === "dateTime" ? "short" : undefined,
          }).format(date)}
        </>
      );
    }
  }
  return <LinkOrText text={text} onFollow={onFollow} />;
}

function LinkOrText({
  text,
  onFollow,
}: {
  text: string;
  onFollow: (target: string) => void;
}) {
  const target = wikilinkTarget(text);
  if (target === null) {
    return <>{text}</>;
  }
  const label = /\|([^\]]+)\]\]$/.exec(text)?.[1] ?? target;
  return (
    <button
      type="button"
      className={styles.link}
      onClick={() => onFollow(target)}
    >
      {label}
    </button>
  );
}

function ValueEditor({
  label,
  kind,
  value,
  suggestions,
  onValue,
}: {
  label: string;
  kind: PropertyKind;
  value: PropertyValueView;
  suggestions: string[];
  onValue: (value: PropertyValueView) => void;
}) {
  const { t } = useTranslation();
  const suggestionsId = useId();
  const text = textOf(value);

  if (kind === "object") {
    return <span className={styles.readonly}>{text}</span>;
  }
  if (kind === "checkbox") {
    return (
      <input
        type="checkbox"
        aria-label={label}
        checked={value.type === "boolean" && value.value}
        onChange={(event) =>
          onValue({ type: "boolean", value: event.currentTarget.checked })
        }
      />
    );
  }
  const datalist = (
    <datalist id={suggestionsId}>
      {suggestions.map((suggestion) => (
        <option key={suggestion} value={suggestion} />
      ))}
    </datalist>
  );
  if (isListKind(kind)) {
    const items = itemsOf(value);
    const setItems = (next: string[]) =>
      onValue({
        type: "list",
        items: next.map((item) => ({ type: "text", value: item })),
      });
    const addItem = (input: HTMLInputElement) => {
      const item = input.value.trim();
      input.value = "";
      if (item !== "" && !items.includes(item)) {
        setItems([...items, item]);
      }
    };
    return (
      <span className={styles.chips}>
        {items.map((item) => (
          <span key={item} className={styles.chip}>
            {item}
            <button
              type="button"
              aria-label={t("properties.removeValue", { value: item })}
              onClick={() => setItems(items.filter((other) => other !== item))}
            >
              ×
            </button>
          </span>
        ))}
        <input
          className={styles.itemInput}
          list={suggestionsId}
          aria-label={t("properties.addValue", { name: label })}
          placeholder={t("properties.addValuePlaceholder")}
          onBlur={(event) => addItem(event.currentTarget)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              addItem(event.currentTarget);
            }
          }}
        />
        {datalist}
      </span>
    );
  }
  const inputType =
    kind === "number"
      ? "number"
      : kind === "date"
        ? "date"
        : kind === "dateTime"
          ? "datetime-local"
          : "text";
  const commit = (raw: string) => {
    if (raw === text) {
      return;
    }
    if (kind === "number" && raw !== "") {
      const number = Number(raw);
      onValue(
        Number.isNaN(number)
          ? textValue(raw)
          : { type: "number", value: number },
      );
    } else {
      onValue(textValue(raw));
    }
  };
  return (
    <>
      <input
        key={text}
        className={styles.value}
        type={inputType}
        defaultValue={kind === "dateTime" ? text.slice(0, 16) : text}
        aria-label={label}
        list={inputType === "text" ? suggestionsId : undefined}
        onBlur={(event) => commit(event.currentTarget.value)}
        onKeyDown={commitOnEnter}
        onChange={(event) => {
          if (inputType === "date" || inputType === "datetime-local") {
            commit(event.currentTarget.value);
          }
        }}
      />
      {inputType === "text" && datalist}
    </>
  );
}

function commitOnEnter(event: KeyboardEvent<HTMLInputElement>) {
  if (event.key === "Enter") {
    event.currentTarget.blur();
  }
}

const ICONS: Record<PropertyKind, string> = {
  text: "≡",
  list: "☰",
  number: "#",
  checkbox: "☑",
  date: "▦",
  dateTime: "◷",
  tags: "⌗",
  aliases: "↪",
  object: "{}",
};
