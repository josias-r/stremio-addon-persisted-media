export function getManifest() {
  return {
    id: "org.stremio.sample.modular",
    version: "1.0.0",
    name: "Modular Node Addon",
    description:
      "A Stremio addon using ES Modules with dynamic content and series.",
    resources: ["stream"],
    types: ["movie", "series"],
    catalogs: [],
  };
}
