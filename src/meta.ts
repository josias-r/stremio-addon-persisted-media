import { setTimeout } from "node:timers/promises";

export type ContentType = "movie" | "series" | "channel" | "tv";

export interface MetaDetail {
  id: string;
  type: ContentType;
  name: string;
  poster?: string;
  background?: string;
  logo?: string;
  description?: string;
  releaseInfo?: string;
  director?: string[];
  cast?: string[];
}

export interface MetaResponse {
  meta: MetaDetail;
}

export async function getMovieMeta(id: string): Promise<MetaResponse> {
  await setTimeout(1000);
  return {
    meta: {
      id,
      type: "movie",
      name: `Dynamic Movie (${id})`,
      poster:
        "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c5/Big_buck_bunny_poster_big.jpg/220px-Big_buck_bunny_poster_big.jpg",
      background:
        "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c5/Big_buck_bunny_poster_big.jpg/800px-Big_buck_bunny_poster_big.jpg",
      description: `This is a dynamically generated movie for ID: ${id}.`,
      releaseInfo: new Date().getFullYear().toString(),
    },
  };
}

export async function getSeriesMeta(id: string): Promise<MetaResponse> {
  await setTimeout(1000);
  return {
    meta: {
      id,
      type: "series",
      name: `Dynamic Series (${id})`,
      poster:
        "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c5/Big_buck_bunny_poster_big.jpg/220px-Big_buck_bunny_poster_big.jpg",
      background:
        "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c5/Big_buck_bunny_poster_big.jpg/800px-Big_buck_bunny_poster_big.jpg",
      description: `This is a dynamically generated series for ID: ${id}.`,
      releaseInfo: new Date().getFullYear().toString(),
    },
  };
}
