import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { useToast } from "../../components/ui/Toast";
import "./categories.css";

type TileVariant =
  | "hero-sky"
  | "hero-blue"
  | "hero-orange"
  | "small-orange"
  | "small-olive"
  | "small-pink"
  | "small-purple"
  | "small-pink-light"
  | "small-black"
  | "small-orange-pale";

type Tile = {
  key: string;
  i18nKey: string;
  variant: TileVariant;
  enabled: boolean;
  path?: string;
};

const WORLD_TILES: Tile[] = [
  { key: "world-map", i18nKey: "categories.world_map", variant: "hero-sky", enabled: true, path: "/world" },
  { key: "webcams", i18nKey: "categories.webcams", variant: "hero-blue", enabled: false },
  { key: "films", i18nKey: "categories.films_together", variant: "hero-orange", enabled: false },
  { key: "live", i18nKey: "categories.live", variant: "small-orange", enabled: false },
  { key: "radio", i18nKey: "categories.radio", variant: "small-olive", enabled: false },
  { key: "tv", i18nKey: "categories.tv", variant: "small-pink", enabled: false },
  { key: "irl", i18nKey: "categories.irl", variant: "small-purple", enabled: false },
];

const CHILL_TILES: Tile[] = [
  { key: "subs", i18nKey: "categories.subs", variant: "small-pink-light", enabled: false },
  { key: "clips", i18nKey: "categories.clips", variant: "small-black", enabled: false },
  { key: "clubs", i18nKey: "categories.clubs", variant: "small-orange-pale", enabled: false },
  { key: "games", i18nKey: "categories.games", variant: "small-orange-pale", enabled: false },
  { key: "singers", i18nKey: "categories.singers", variant: "small-orange-pale", enabled: false },
  { key: "cartoons", i18nKey: "categories.cartoons", variant: "small-orange-pale", enabled: false },
];

export default function CategoriesScreen() {
  const { t } = useTranslation();
  const nav = useNavigate();
  const { show } = useToast();

  const handleClick = (tile: Tile) => {
    if (tile.enabled && tile.path) {
      nav(tile.path);
    } else {
      show(t("categories.coming_soon"));
    }
  };

  const renderTile = (tile: Tile) => (
    <button
      key={tile.key}
      type="button"
      className={`cats__tile cats__tile--${tile.variant}${tile.enabled ? "" : " cats__tile--disabled"}`}
      onClick={() => handleClick(tile)}
      aria-disabled={!tile.enabled}
    >
      <span className="cats__tile-title">{t(tile.i18nKey)}</span>
    </button>
  );

  return (
    <div className="cats">
      <h2 className="cats__section">{t("categories.world_section")}</h2>
      <div className="cats__world">{WORLD_TILES.map(renderTile)}</div>

      <h2 className="cats__section">{t("categories.chill_section")}</h2>
      <div className="cats__chill">{CHILL_TILES.map(renderTile)}</div>
    </div>
  );
}
