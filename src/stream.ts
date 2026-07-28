import { fetchJackettResults, type JackettResult } from "./jackett.ts";
import { getReqEnvVariable } from "./loadenv.ts";
import {
  getTorrentsByStremioId,
  getTorrentByHash,
  linkTorrentToStremioId,
  type TorrentRecord,
} from "./db.ts";
import { getTorrentFiles } from "./qbittorrent.ts";

interface Subtitle {
  id: string;
  url: string;
  lang: string;
}

interface StreamBehaviorHints {
  notWebReady?: boolean;
  bingeGroup?: string;
  proxyHeaders?: {
    request?: Record<string, string>;
    response?: Record<string, string>;
  };
}

interface StreamBase {
  name?: string;
  title?: string;
  description?: string;
  subtitles?: Subtitle[];
  behaviorHints?: StreamBehaviorHints;
}

interface UrlStream extends StreamBase {
  url: string;
}

interface YtStream extends StreamBase {
  ytId: string;
}

interface TorrentStream extends StreamBase {
  infoHash: string;
  fileIdx?: number;
  sources?: string[];
}

interface ExternalStream extends StreamBase {
  externalUrl: string;
}

interface PlayerFrameStream extends StreamBase {
  playerFrameUrl: string;
}

export type Stream =
  | UrlStream
  | YtStream
  | TorrentStream
  | ExternalStream
  | PlayerFrameStream;

interface StreamResponse {
  streams: Stream[];
}

const PUBLIC_URL = getReqEnvVariable("PUBLIC_URL");

function extractInfoHash(magnetUri: string | undefined): string | null {
  if (!magnetUri) return null;
  const match = magnetUri.match(/xt=urn:btih:([a-zA-Z0-9]+)/);
  return match ? match[1].toLowerCase() : null;
}

function mapJackettToStream(
  result: JackettResult,
  stremioId: string,
  downloadedHashes: Set<string>,
): Stream | null {
  const magnetOrUrl = result.MagnetUri || result.Link;
  if (!magnetOrUrl) return null;

  const infoHash = (
    result.InfoHash || extractInfoHash(result.MagnetUri)
  )?.toLowerCase();
  if (!infoHash) return null;

  const sizeGB = (result.Size / 1024 / 1024 / 1024).toFixed(2);
  const isDownloading = downloadedHashes.has(infoHash);

  const titlePrefix = isDownloading ? "[Downloading/Downloaded]\n" : "";
  const titleStr = `${titlePrefix}${result.Title}\n👤 ${result.Seeders} Seeders | 💾 ${sizeGB} GB`;

  return {
    name: `Qbit - ${result.Tracker}`,
    title: titleStr,
    url: `${PUBLIC_URL}/trigger-download/${encodeURIComponent(stremioId)}/${infoHash}?magnet=${encodeURIComponent(magnetOrUrl)}`,
  } as UrlStream;
}

const VIDEO_EXTENSIONS = [".mp4", ".mkv", ".avi", ".webm", ".mov"];

async function getLocalFileStreams(
  torrents: TorrentRecord[],
): Promise<Stream[]> {
  const streams: Stream[] = [];

  for (const torrent of torrents) {
    const files = await getTorrentFiles(torrent.infoHash);

    for (const file of files) {
      const ext = file.name.substring(file.name.lastIndexOf(".")).toLowerCase();
      if (VIDEO_EXTENSIONS.includes(ext)) {
        const sizeGB = (file.size / 1024 / 1024 / 1024).toFixed(2);
        const progressPct = Math.round(file.progress * 100);

        streams.push({
          name: "Local Stream",
          title: `${file.name}\nProgress: ${progressPct}% | 💾 ${sizeGB} GB`,
          url: `${PUBLIC_URL}/stream-file/${torrent.infoHash}?filePath=${encodeURIComponent(file.name)}`,
        } as UrlStream);
      }
    }
  }

  return streams;
}

async function buildStreamResponse(
  stremioId: string,
  jackettQuery: string,
): Promise<StreamResponse> {
  let dbTorrents = getTorrentsByStremioId(stremioId);

  // If no local streams exist for this specific request,
  // we fetch Jackett and see if any existing torrent hashes match, automatically linking them.
  const results = await fetchJackettResults(jackettQuery);

  if (dbTorrents.length === 0) {
    let linkedNew = false;
    for (const r of results) {
      const infoHash = (
        r.InfoHash || extractInfoHash(r.MagnetUri)
      )?.toLowerCase();
      if (infoHash && getTorrentByHash(infoHash)) {
        linkTorrentToStremioId(infoHash, stremioId);
        linkedNew = true;
      }
    }
    if (linkedNew) {
      dbTorrents = getTorrentsByStremioId(stremioId);
    }
  }

  const localStreams = await getLocalFileStreams(dbTorrents);
  const downloadedHashes = new Set(
    dbTorrents.map((t) => t.infoHash.toLowerCase()),
  );

  const jackettStreams = results
    .map((r) => mapJackettToStream(r, stremioId, downloadedHashes))
    .filter((s): s is Stream => s !== null);

  return { streams: [...localStreams, ...jackettStreams] };
}

export async function getMovieStream(id: string): Promise<StreamResponse> {
  return buildStreamResponse(id, id);
}

export async function getSeriesStream(
  seriesId: string,
  season: number,
  episode: number,
): Promise<StreamResponse> {
  const stremioId = `${seriesId}:${season}:${episode}`;
  const query = `${seriesId} S${season.toString().padStart(2, "0")}E${episode.toString().padStart(2, "0")}`;

  return buildStreamResponse(stremioId, query);
}
