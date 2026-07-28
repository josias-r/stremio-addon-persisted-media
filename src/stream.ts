import {
  fetchJackettResults,
  type JackettResult,
  type TorznabParams,
} from "./jackett.ts";
import { getReqEnvVariable } from "./loadenv.ts";
import {
  getTorrentsByStremioId,
  getTorrentByHash,
  linkTorrentToStremioId,
  type TorrentRecord,
} from "./db.ts";
import { getTorrentFiles } from "./qbittorrent.ts";
import {
  isDefinitelyWrongEpisode,
  isDefinitelyNotFullSeason,
} from "./parse-episode.ts";
import { getCinemetaTitle } from "./cinemeta.ts";

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
const JACKETT_SEARCH_TYPE = getReqEnvVariable("JACKETT_SEARCH_TYPE");

if (JACKETT_SEARCH_TYPE !== "imdb" && JACKETT_SEARCH_TYPE !== "text") {
  throw new Error(`JACKETT_SEARCH_TYPE must be either "imdb" or "text"`);
}

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
  isDefinitelyNotWanted?: (title: string) => boolean,
): Promise<Stream[]> {
  const streams: Stream[] = [];

  for (const torrent of torrents) {
    const files = await getTorrentFiles(torrent.infoHash);

    for (const file of files) {
      const ext = file.name.substring(file.name.lastIndexOf(".")).toLowerCase();
      if (VIDEO_EXTENSIONS.includes(ext)) {
        // Robust filtering for local files (useful for season packs)
        if (isDefinitelyNotWanted && isDefinitelyNotWanted(file.name)) {
          continue; // Skip files that are clearly not what we want
        }

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

interface StreamFetchPlan {
  params: TorznabParams;
  jackettFilter?: (title: string) => boolean;
}

async function buildStreamResponse(
  stremioId: string,
  fetchPlans: StreamFetchPlan[],
  localFileFilter?: (title: string) => boolean,
): Promise<StreamResponse> {
  let dbTorrents = getTorrentsByStremioId(stremioId);

  // If no local streams exist for this specific request,
  // we fetch Jackett and see if any existing torrent hashes match, automatically linking them.
  const resultsArrays = await Promise.all(
    fetchPlans.map(async (plan) => {
      let planResults = await fetchJackettResults(plan.params);
      if (plan.jackettFilter) {
        planResults = planResults.filter((r) => !plan.jackettFilter!(r.Title));
      }
      return planResults;
    }),
  );
  let results = resultsArrays.flat();

  // Deduplicate Jackett results from concurrent searches
  const seen = new Set<string>();
  results = results.filter((r) => {
    const key = (r.InfoHash || r.MagnetUri || r.Link || r.Title).toLowerCase();
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });

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

  const localStreams = await getLocalFileStreams(dbTorrents, localFileFilter);
  const downloadedHashes = new Set(
    dbTorrents.map((t) => t.infoHash.toLowerCase()),
  );

  const jackettStreams = results
    .map((r) => mapJackettToStream(r, stremioId, downloadedHashes))
    .filter((s): s is Stream => s !== null);

  return { streams: [...localStreams, ...jackettStreams] };
}

export async function getMovieStream(id: string): Promise<StreamResponse> {
  let fetchPlans: StreamFetchPlan[] = [
    { params: { type: "movie_imdb", imdbId: id } },
  ];

  if (JACKETT_SEARCH_TYPE === "text") {
    const title = await getCinemetaTitle("movie", id);
    if (title) {
      fetchPlans = [{ params: { type: "movie_text", query: title } }];
    }
  }

  return buildStreamResponse(id, fetchPlans);
}

export async function getSeriesStream(
  seriesId: string,
  season: number,
  episode: number,
): Promise<StreamResponse> {
  const stremioId = `${seriesId}:${season}:${episode}`;

  let fetchPlans: StreamFetchPlan[] = [
    {
      params: { type: "series_imdb", imdbId: seriesId, season, episode },
      jackettFilter: (title) =>
        isDefinitelyWrongEpisode(title, season, episode),
    },
    {
      params: { type: "series_season_imdb", imdbId: seriesId, season },
      jackettFilter: (title) => isDefinitelyNotFullSeason(title, season),
    },
  ];

  if (JACKETT_SEARCH_TYPE === "text") {
    const title = await getCinemetaTitle("series", seriesId);
    if (title) {
      fetchPlans = [
        {
          params: { type: "series_text", query: title, season, episode },
          jackettFilter: (title) =>
            isDefinitelyWrongEpisode(title, season, episode),
        },
        {
          params: { type: "series_season_text", query: title, season },
          jackettFilter: (title) => isDefinitelyNotFullSeason(title, season),
        },
      ];
    }
  }

  return buildStreamResponse(stremioId, fetchPlans, (title: string) =>
    isDefinitelyWrongEpisode(title, season, episode),
  );
}
