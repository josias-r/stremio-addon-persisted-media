import { XMLParser } from "fast-xml-parser";
import { getReqEnvVariable } from "./loadenv.ts";

const JACKETT_URL = getReqEnvVariable("JACKETT_URL");
const JACKETT_API_KEY = getReqEnvVariable("JACKETT_API_KEY");

if (!JACKETT_URL) {
  throw new Error("JACKETT_URL environment variable is required.");
}
if (!JACKETT_API_KEY) {
  throw new Error("JACKETT_API_KEY environment variable is required.");
}

export interface JackettResult {
  Title: string;
  Tracker: string;
  TrackerId: string;
  Link?: string;
  MagnetUri?: string;
  InfoHash?: string;
  Seeders: number;
  Peers: number;
  Size: number;
}

export type TorznabParams =
  | { type: "movie"; imdbId: string }
  | { type: "series"; imdbId: string; season: number; episode: number };

function buildTorznabQueryString(params: TorznabParams): string {
  const queryParams = new URLSearchParams();
  queryParams.append("apikey", JACKETT_API_KEY as string);

  if (params.type === "movie") {
    queryParams.append("t", "movie");
    queryParams.append("imdbid", params.imdbId);
  } else if (params.type === "series") {
    queryParams.append("t", "tvsearch");
    queryParams.append("imdbid", params.imdbId);
    queryParams.append("season", params.season.toString());
    queryParams.append("ep", params.episode.toString());
  }

  return queryParams.toString();
}

export async function fetchJackettResults(
  torznabParams: TorznabParams,
): Promise<JackettResult[]> {
  const qs = buildTorznabQueryString(torznabParams);
  const url = `${JACKETT_URL}/api/v2.0/indexers/all/results/torznab/api?${qs}`;

  try {
    const response = await fetch(url);
    if (!response.ok) {
      console.error(`Jackett API responded with status: ${response.status}`);
      return [];
    }

    const xml = await response.text();
    const parser = new XMLParser({
      ignoreAttributes: false,
      attributeNamePrefix: "@_",
      isArray: (name) => name === "item" || name === "torznab:attr",
    });
    const jsonObj = parser.parse(xml);
    const items = jsonObj.rss?.channel?.item || [];
    const results: JackettResult[] = [];

    for (const item of items) {
      const title = item.title ? String(item.title) : "Unknown";
      const size = parseInt(item.size, 10) || 0;
      const link = item.link;

      const enclosureUrl = item.enclosure && item.enclosure["@_url"];
      const magnetUri =
        enclosureUrl && enclosureUrl.startsWith("magnet:")
          ? enclosureUrl
          : link;

      let seeders = 0;
      let infoHash = undefined;

      const attrs = item["torznab:attr"] || [];
      for (const attr of attrs) {
        if (attr["@_name"] === "seeders") {
          seeders = parseInt(attr["@_value"], 10) || 0;
        }
        if (attr["@_name"] === "infohash") {
          infoHash = attr["@_value"];
        }
      }

      let trackerId = "";
      let trackerName = "Unknown";

      if (item.jackettindexer) {
        if (typeof item.jackettindexer === "object") {
          trackerId = item.jackettindexer["@_id"] || "";
          trackerName = item.jackettindexer["#text"] || "Unknown";
        } else {
          trackerName = String(item.jackettindexer);
        }
      }

      if (seeders > 0) {
        results.push({
          Title: title,
          Tracker: trackerName,
          TrackerId: trackerId,
          Link: magnetUri,
          MagnetUri: magnetUri,
          InfoHash: infoHash,
          Seeders: seeders,
          Peers: 0,
          Size: size,
        });
      }
    }

    return results.sort((a, b) => (b.Seeders || 0) - (a.Seeders || 0));
  } catch (error) {
    console.error("Failed to fetch from Jackett API:", error);
    return [];
  }
}
