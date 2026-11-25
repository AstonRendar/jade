import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import styles from "./ImageLightbox.module.css";

export function ImageLightbox({
  src,
  alt,
  onClose,
}: {
  src: string;
  alt: string;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  useEffect(() => {
    const close = (event: KeyboardEvent) => event.key === "Escape" && onClose();
    window.addEventListener("keydown", close);
    return () => window.removeEventListener("keydown", close);
  }, [onClose]);
  return (
    <div
      role="dialog"
      aria-label={alt || t("images.viewer")}
      className={styles.lightbox}
      onClick={onClose}
    >
      <img src={src} alt={alt} />
    </div>
  );
}
