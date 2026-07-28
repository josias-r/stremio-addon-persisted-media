import http from "node:http";
import { getManifest } from "./src/manifest.ts";
import { getMovieStream, getSeriesStream } from "./src/stream.ts";
import { addTorrentToQbit } from "./src/qbittorrent.ts";
import { getOptionalEnvVariable } from "./src/loadenv.ts";

const PORT = getOptionalEnvVariable("PORT") || 3000;

// Helper to send JSON responses with CORS headers required by Stremio
function sendJson(res: http.ServerResponse, data: unknown) {
  res.writeHead(200, {
    "Content-Type": "application/json; charset=utf-8",
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Headers": "*",
  });
  res.end(JSON.stringify(data));
}

// Helper to send 404 Error with CORS headers
function send404(res: http.ServerResponse) {
  res.writeHead(404, {
    "Access-Control-Allow-Origin": "*",
    "Access-Control-Allow-Headers": "*",
  });
  res.end("Not Found");
}

const server = http.createServer(async (req, res) => {
  // Handle CORS preflight requests
  if (req.method === "OPTIONS") {
    res.writeHead(200, {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Headers": "*",
    });
    res.end();
    return;
  }

  const url = req.url || "/";
  console.log(`[${req.method}] ${url}`);

  // Endpoint: Manifest
  if (url === "/manifest.json") {
    return sendJson(res, getManifest());
  }

  const streamMatch = url.match(/^\/stream\/(movie|series)\/([^/]+)\.json$/);
  if (streamMatch) {
    const [, type, idEncoded] = streamMatch;
    const decodedId = decodeURIComponent(idEncoded);

    if (type === "movie") {
      return sendJson(res, await getMovieStream(decodedId));
    } else {
      // Series stream IDs typically look like tt1234567:1:1 (id:season:episode)
      const [seriesId, seasonStr, episodeStr] = decodedId.split(":");
      const season = parseInt(seasonStr || "1", 10);
      const episode = parseInt(episodeStr || "1", 10);

      return sendJson(res, await getSeriesStream(seriesId, season, episode));
    }
  }

  const addTorrentMatch = url.match(/^\/add-torrent\/(.+)$/);
  if (addTorrentMatch) {
    const magnetUri = decodeURIComponent(addTorrentMatch[1]);
    const success = await addTorrentToQbit(magnetUri);

    if (success) {
      res.writeHead(200, { "Content-Type": "text/plain" });
      res.end("Successfully added to qBittorrent.");
    } else {
      res.writeHead(500, { "Content-Type": "text/plain" });
      res.end("Failed to add to qBittorrent.");
    }
    return;
  }

  return send404(res);
});

server.listen(PORT, () => {
  console.log(
    `Modular Stremio Add-on server is running at http://localhost:${PORT}`,
  );
  console.log(`Manifest URL: http://localhost:${PORT}/manifest.json`);
});
