import { StateEffect, StateField, type EditorState } from "@codemirror/state";
import {
  Decoration,
  EditorView,
  WidgetType,
  type DecorationSet,
} from "@codemirror/view";
import { imageEmbedsIn } from "../images/imageFiles";

type Urls = Record<string, string | null>;

export const setImageUrls = StateEffect.define<Urls>();

const imageUrls = StateField.define<Urls>({
  create: () => ({}),
  update: (urls, transaction) =>
    transaction.effects.reduce(
      (current, effect) => (effect.is(setImageUrls) ? effect.value : current),
      urls,
    ),
});

class ImageWidget extends WidgetType {
  constructor(
    readonly url: string,
    readonly width: number | undefined,
  ) {
    super();
  }

  eq(other: ImageWidget): boolean {
    return other.url === this.url && other.width === this.width;
  }

  toDOM(): HTMLElement {
    const wrapper = document.createElement("div");
    wrapper.className = "cm-image";
    const image = document.createElement("img");
    image.src = this.url;
    image.alt = "";
    if (this.width) {
      image.width = this.width;
    }
    wrapper.append(image);
    return wrapper;
  }
}

function widgets(state: EditorState): DecorationSet {
  const urls = state.field(imageUrls);
  const ranges = [];
  for (let number = 1; number <= state.doc.lines; number += 1) {
    const line = state.doc.line(number);
    for (const embed of imageEmbedsIn(line.text)) {
      const url = urls[embed.target];
      if (url) {
        ranges.push(
          Decoration.widget({
            widget: new ImageWidget(url, embed.width),
            block: true,
            side: 1,
          }).range(line.to),
        );
      }
    }
  }
  return Decoration.set(ranges);
}

const imageDecorations = StateField.define<DecorationSet>({
  create: widgets,
  update: (decorations, transaction) =>
    transaction.docChanged ||
    transaction.effects.some((effect) => effect.is(setImageUrls))
      ? widgets(transaction.state)
      : decorations,
  provide: (field) => EditorView.decorations.from(field),
});

export const imagePreview = [imageUrls, imageDecorations];
