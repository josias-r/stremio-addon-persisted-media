import { fetchJackettResults, type JackettResult } from "./jackett.ts";
import { getReqEnvVariable } from "./loadenv.ts";
import { getTorrentsByStremioId } from "./db.ts";
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

async function getLocalFileStreams(stremioId: string): Promise<Stream[]> {
  const torrents = getTorrentsByStremioId(stremioId);
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

export async function getMovieStream(id: string): Promise<StreamResponse> {
  const localStreams = await getLocalFileStreams(id);
  const dbTorrents = getTorrentsByStremioId(id);
  const downloadedHashes = new Set(
    dbTorrents.map((t) => t.infoHash.toLowerCase()),
  );

  const results = await fetchJackettResults(id);
  const jackettStreams = results
    .map((r) => mapJackettToStream(r, id, downloadedHashes))
    .filter((s): s is Stream => s !== null);

  return { streams: [...localStreams, ...jackettStreams] };
}

export async function getSeriesStream(
  seriesId: string,
  season: number,
  episode: number,
): Promise<StreamResponse> {
  const stremioId = `${seriesId}:${season}:${episode}`;
  const localStreams = await getLocalFileStreams(stremioId);

  const dbTorrents = getTorrentsByStremioId(stremioId);
  const downloadedHashes = new Set(
    dbTorrents.map((t) => t.infoHash.toLowerCase()),
  );

  const query = `${seriesId} S${season.toString().padStart(2, "0")}E${episode.toString().padStart(2, "0")}`;
  const results = await fetchJackettResults(query);
  const jackettStreams = results
    .map((r) => mapJackettToStream(r, stremioId, downloadedHashes))
    .filter((s): s is Stream => s !== null);

  return { streams: [...localStreams, ...jackettStreams] };
}
