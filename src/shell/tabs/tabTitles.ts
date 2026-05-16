/**
 * Given a path, return a human-readable tab title.
 * Uses the first segment for section names; deeper paths
 * can be made more specific later (e.g. /room/:id -> room name).
 */
export function titleForPath(path: string, tFn: (key: string) => string): string {
  const clean = path.split("?")[0].split("#")[0].replace(/^\/+/, "");
  const [head, ...rest] = clean.split("/");
  switch (head) {
    case "":
    case "home":
      return tFn("shell.tab.home");
    case "categories":
      return tFn("shell.tab.categories");
    case "world":
      return tFn("shell.tab.world");
    case "create":
      return tFn("shell.tab.create");
    case "people":
      return tFn("shell.tab.people");
    case "notifications":
      return tFn("shell.tab.notifications");
    case "history":
      return tFn("shell.tab.history");
    case "favorites":
      return tFn("shell.tab.favorites");
    case "folders":
      return tFn("shell.tab.folders");
    case "settings":
      return tFn("shell.tab.settings");
    case "profile":
      // path: /profile/:username -> show username if present
      return rest[0] ? `@${rest[0]}` : tFn("shell.tab.profile");
    case "room":
      return rest[0] ? tFn("shell.tab.room") + " " + rest[0].slice(0, 6) : tFn("shell.tab.room");
    case "welcome":
    case "login":
    case "register":
      return tFn("shell.tab.auth");
    default:
      return clean || tFn("shell.tab.home");
  }
}
