export function getManifest() {
  return {
    id: "mini-media-server.addon",
    version: "1.0.0",
    name: "Mini Media Server",
    description:
      "A tiny Stremio addon acting similar to a self-hostable debrid service. Search for torrents via Jackett, download and cache them automatically and serve them to Stremio.",
    resources: ["stream"],
    types: ["movie", "series"],
    catalogs: [],
  };
}
