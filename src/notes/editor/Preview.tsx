import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useMemo, useState, type MouseEvent } from "react";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { imageEmbedsIn } from "../images/imageFiles";
import { ImageLightbox } from "../images/ImageLightbox";
import { useImages } from "../store/imageStore";
import styles from "./NoteView.module.css";
import { renderMarkdown } from "./renderMarkdown";

type PreviewProps = {
  source: string;
  onFollow: (target: string) => void;
};

export function Preview({ source, onFollow }: PreviewProps) {
  const urls = useImages((state) => state.urls);
  const [enlarged, setEnlarged] = useState<{ src: string; alt: string } | null>(
    null,
  );
  const html = useMemo(() => renderMarkdown(source, urls), [source, urls]);

  useEffect(() => {
    const targets = source.split("\n").flatMap(imageEmbedsIn);
    if (targets.length > 0) {
      void reportingErrors(() =>
        useImages.getState().resolve(targets.map((embed) => embed.target)),
      );
    }
  }, [source]);

  const handleClick = (event: MouseEvent<HTMLDivElement>) => {
    const element = event.target as HTMLElement;
    if (element instanceof HTMLImageElement) {
      setEnlarged({ src: element.src, alt: element.alt });
      return;
    }
    const anchor = element.closest("a");
    if (!anchor) {
      return;
    }
    event.preventDefault();
    const target = anchor.dataset.target;
    if (target) {
      onFollow(target);
    } else if (/^https?:\/\//.test(anchor.href)) {
      void openUrl(anchor.href);
    }
  };
  return (
    <>
      <article
        className={styles.preview}
        onClick={handleClick}
        dangerouslySetInnerHTML={{ __html: html }}
      />
      {enlarged && (
        <ImageLightbox
          src={enlarged.src}
          alt={enlarged.alt}
          onClose={() => setEnlarged(null)}
        />
      )}
    </>
  );
}
