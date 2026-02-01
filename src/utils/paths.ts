import { homeDir, join } from "@tauri-apps/api/path";

export async function resolveDefaultPath(path: string | null | undefined) {
  if (!path || path.trim().length === 0) return undefined;
  if (path === "~") {
    return homeDir();
  }
  if (path.startsWith("~/")) {
    const home = await homeDir();
    return join(home, path.slice(2));
  }
  return path;
}
