import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { getReqEnvVariable } from "./loadenv.ts";

const DOWNLOAD_PATH = getReqEnvVariable("DOWNLOAD_PATH");

export function handleStreamFile(
  req: http.IncomingMessage,
  res: http.ServerResponse,
  filePathParam: string,
) {
  const fullPath = path.join(DOWNLOAD_PATH, filePathParam);

  // Check for directory traversal attacks
  if (!fullPath.startsWith(DOWNLOAD_PATH)) {
    res.writeHead(404);
    res.end("Not Found");
    return;
  }

  if (!fs.existsSync(fullPath)) {
    res.writeHead(404);
    res.end("Not Found");
    return;
  }

  const stat = fs.statSync(fullPath);
  const fileSize = stat.size;
  const range = req.headers.range;

  if (range) {
    const parts = range.replace(/bytes=/, "").split("-");
    const start = parseInt(parts[0], 10);
    const end = parts[1] ? parseInt(parts[1], 10) : fileSize - 1;
    const chunksize = end - start + 1;
    const file = fs.createReadStream(fullPath, { start, end });
    const head = {
      "Content-Range": `bytes ${start}-${end}/${fileSize}`,
      "Accept-Ranges": "bytes",
      "Content-Length": chunksize,
      "Content-Type": "video/mp4", // Or detect dynamically
      "Access-Control-Allow-Origin": "*",
    };
    res.writeHead(206, head);
    file.pipe(res);
  } else {
    const head = {
      "Content-Length": fileSize,
      "Content-Type": "video/mp4",
      "Access-Control-Allow-Origin": "*",
    };
    res.writeHead(200, head);
    fs.createReadStream(fullPath).pipe(res);
  }
}
