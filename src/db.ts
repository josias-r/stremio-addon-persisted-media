import { DatabaseSync } from "node:sqlite";
import { getOptionalEnvVariable } from "./loadenv.ts";

const dbPath = getOptionalEnvVariable("DB_PATH") || "torrents.db";
const db = new DatabaseSync(dbPath);

// Initialize tables
db.exec(`
  CREATE TABLE IF NOT EXISTS torrents (
    infoHash TEXT PRIMARY KEY,
    magnetUri TEXT NOT NULL
  );

  CREATE TABLE IF NOT EXISTS torrent_streams (
    infoHash TEXT NOT NULL,
    stremioId TEXT NOT NULL,
    PRIMARY KEY (infoHash, stremioId),
    FOREIGN KEY (infoHash) REFERENCES torrents (infoHash)
  );
`);

export interface TorrentRecord {
  infoHash: string;
  magnetUri: string;
}

export function addTorrent(
  infoHash: string,
  stremioId: string,
  magnetUri: string,
) {
  const insertTorrent = db.prepare(
    "INSERT OR IGNORE INTO torrents (infoHash, magnetUri) VALUES (?, ?)",
  );
  insertTorrent.run(infoHash, magnetUri);

  linkTorrentToStremioId(infoHash, stremioId);
}

export function linkTorrentToStremioId(infoHash: string, stremioId: string) {
  const insertLink = db.prepare(
    "INSERT OR IGNORE INTO torrent_streams (infoHash, stremioId) VALUES (?, ?)",
  );
  insertLink.run(infoHash, stremioId);
}

export function getTorrentsByStremioId(stremioId: string): TorrentRecord[] {
  const stmt = db.prepare(`
    SELECT t.* FROM torrents t
    JOIN torrent_streams ts ON t.infoHash = ts.infoHash
    WHERE ts.stremioId = ?
  `);
  return stmt.all(stremioId) as unknown as TorrentRecord[];
}

export function getTorrentByHash(infoHash: string): TorrentRecord | undefined {
  const stmt = db.prepare("SELECT * FROM torrents WHERE infoHash = ?");
  return stmt.get(infoHash) as TorrentRecord | undefined;
}
