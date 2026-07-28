import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { addTorrentToQbit } from "./qbittorrent.ts";
import { addTorrent } from "./db.ts";

const DL_PLACEHOLDER_VIDEO = path.join(process.cwd(), "placeholder.mp4");

export async function handleTriggerDownload(
  res: http.ServerResponse,
  urlObj: URL,
  triggerMatch: RegExpMatchArray,
) {
  const stremioId = decodeURIComponent(triggerMatch[1]);
  const infoHash = triggerMatch[2];
  const magnetUri = urlObj.searchParams.get("magnet");

  if (magnetUri) {
    addTorrent(infoHash, stremioId, magnetUri);
    await addTorrentToQbit(magnetUri);
  }

  // Serve a tiny dummy mp4 or empty 200 response to satisfy the player temporarily
  res.writeHead(200, { "Content-Type": "video/mp4" });
  fs.createReadStream(DL_PLACEHOLDER_VIDEO).pipe(res);
}
