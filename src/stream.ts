import type { ContentType } from "./meta.ts";
import { setTimeout } from "node:timers/promises";

export interface Subtitle {
  id: string;
  url: string;
  lang: string;
}

export interface StreamBehaviorHints {
  notWebReady?: boolean;
  bingeGroup?: string;
  proxyHeaders?: {
    request?: Record<string, string>;
    response?: Record<string, string>;
  };
}

export interface StreamBase {
  name?: string;
  title?: string;
  description?: string;
  subtitles?: Subtitle[];
  behaviorHints?: StreamBehaviorHints;
}

export interface UrlStream extends StreamBase {
  url: string;
}

export interface YtStream extends StreamBase {
  ytId: string;
}

export interface TorrentStream extends StreamBase {
  infoHash: string;
  fileIdx?: number;
  sources?: string[];
}

export interface ExternalStream extends StreamBase {
  externalUrl: string;
}

export interface PlayerFrameStream extends StreamBase {
  playerFrameUrl: string;
}

export type Stream =
  | UrlStream
  | YtStream
  | TorrentStream
  | ExternalStream
  | PlayerFrameStream;

export interface StreamResponse {
  streams: Stream[];
}

export async function getMovieStream(id: string): Promise<StreamResponse> {
  await setTimeout(1000);
  return {
    streams: [
      {
        name: "1080p Movie Server",
        title: `Big Buck Bunny Movie - ${id}`,
        url: "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
      },
      {
        name: "720p Movie Server",
        title: `Elephant's Dream Movie - ${id}`,
        url: "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4",
      },
    ],
  };
}

export async function getSeriesStream(
  seriesId: string,
  season: number,
  episode: number,
): Promise<StreamResponse> {
  await setTimeout(5000);
  return {
    streams: [
      {
        name: "1080p Series Server",
        title: `Episode S${season.toString().padStart(2, "0")}E${episode.toString().padStart(2, "0")} - ${seriesId}`,
        url: "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
      },
      {
        name: "720p Series Server",
        title: `Alternative Stream - S${season.toString().padStart(2, "0")}E${episode.toString().padStart(2, "0")}`,
        url: "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4",
      },
    ],
  };
}
