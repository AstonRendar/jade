import { useEffect } from "react";
import { reportingErrors } from "../../shared/ui/notificationStore";
import { useImages } from "../store/imageStore";
import styles from "./ImageView.module.css";

export function ImageView({ path }: { path: string }) {
  const url = useImages((state) => state.urls[path]);
  const name = path.slice(path.lastIndexOf("/") + 1);

  useEffect(() => {
    void reportingErrors(() => useImages.getState().resolve([path]));
  }, [path]);

  return (
    <section className={styles.view}>
      <header className={styles.header}>
        <h1>{name}</h1>
      </header>
      <div className={styles.canvas}>{url && <img src={url} alt={name} />}</div>
    </section>
  );
}
