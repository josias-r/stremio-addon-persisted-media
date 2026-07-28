import { fetchJackettResults, type JackettResult } from "./jackett.ts";
import { getReqEnvVariable } from "./loadenv.ts";

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

type Stream =
  | UrlStream
  | YtStream
  | TorrentStream
  | ExternalStream
  | PlayerFrameStream;

interface StreamResponse {
  streams: Stream[];
}

const PUBLIC_URL = getReqEnvVariable("PUBLIC_URL");

function mapJackettToStream(result: JackettResult): Stream | null {
  // Fallback to Link if MagnetUri is not provided by the tracker
  const magnetOrUrl = result.MagnetUri || result.Link;

  if (!magnetOrUrl) return null;

  const sizeGB = (result.Size / 1024 / 1024 / 1024).toFixed(2);
  const titleStr = `${result.Title}\n👤 ${result.Seeders} Seeders | 💾 ${sizeGB} GB`;

  return {
    name: `Qbit - ${result.Tracker}`,
    title: titleStr,
    url: `${PUBLIC_URL}/add-torrent/${encodeURIComponent(magnetOrUrl)}`,
  } as UrlStream;
}

export async function getMovieStream(id: string): Promise<StreamResponse> {
  const results = await fetchJackettResults(id);
  const streams = results
    .map(mapJackettToStream)
    .filter((s): s is Stream => s !== null);

  return { streams };
}

export async function getSeriesStream(
  seriesId: string,
  season: number,
  episode: number,
): Promise<StreamResponse> {
  const query = `${seriesId} S${season.toString().padStart(2, "0")}E${episode.toString().padStart(2, "0")}`;
  const results = await fetchJackettResults(query);
  const streams = results
    .map(mapJackettToStream)
    .filter((s): s is Stream => s !== null);

  return { streams };
}
