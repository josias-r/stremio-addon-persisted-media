import { fetchJackettResults, type JackettResult } from "./jackett.ts";

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

function mapJackettToStream(result: JackettResult): Stream | null {
  let infoHash = result.InfoHash;

  if (!infoHash && result.MagnetUri) {
    const match = result.MagnetUri.match(/urn:btih:([a-zA-Z0-9]+)/i);
    if (match) {
      infoHash = match[1].toLowerCase();
    }
  }

  const sizeGB = (result.Size / 1024 / 1024 / 1024).toFixed(2);
  const titleStr = `${result.Title}\n👤 ${result.Seeders} Seeders | 💾 ${sizeGB} GB`;

  if (infoHash) {
    return {
      name: `Jackett - ${result.Tracker}`,
      title: titleStr,
      infoHash,
    } as TorrentStream;
  }

  if (result.Link && result.Link.startsWith("http")) {
    return {
      name: `Jackett - ${result.Tracker}`,
      title: titleStr,
      url: result.Link,
    } as UrlStream;
  }

  return null;
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
