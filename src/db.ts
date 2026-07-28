import { DatabaseSync } from "node:sqlite";

const db = new DatabaseSync("torrents.db");

// Initialize table
db.exec(`
  CREATE TABLE IF NOT EXISTS torrents (
    infoHash TEXT PRIMARY KEY,
    stremioId TEXT NOT NULL,
    magnetUri TEXT NOT NULL
  )
`);

export interface TorrentRecord {
  infoHash: string;
  stremioId: string;
  magnetUri: string;
}

export function addTorrent(
  infoHash: string,
  stremioId: string,
  magnetUri: string,
) {
  const stmt = db.prepare(
    "INSERT OR IGNORE INTO torrents (infoHash, stremioId, magnetUri) VALUES (?, ?, ?)",
  );
  stmt.run(infoHash, stremioId, magnetUri);
}

export function getTorrentsByStremioId(stremioId: string): TorrentRecord[] {
  // Extract base series ID if it's an episode (e.g., tt1234567:1:1 -> tt1234567)
  const baseId = stremioId.split(":")[0];

  // Find all torrents that start with the baseId
  // This allows season packs downloaded for S01E01 to also appear for S01E02
  const stmt = db.prepare("SELECT * FROM torrents WHERE stremioId LIKE ?");
  return stmt.all(`${baseId}%`) as unknown as TorrentRecord[];
}

export function getTorrentByHash(infoHash: string): TorrentRecord | undefined {
  const stmt = db.prepare("SELECT * FROM torrents WHERE infoHash = ?");
  return stmt.get(infoHash) as TorrentRecord | undefined;
}
