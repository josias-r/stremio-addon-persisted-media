import http from "node:http";

// Native TypeScript interfaces directly in Node!
interface ResponseData {
  status: string;
  message: string;
  timestamp: string;
}

const PORT = process.env.PORT || 3000;

const server = http.createServer((req, res) => {
  const data: ResponseData = {
    status: "success",
    message: "Zero-dependency TypeScript in Node.js!",
    timestamp: new Date().toISOString(),
  };

  res.writeHead(200, { "Content-Type": "application/json" });
  res.end(JSON.stringify(data));
});

server.listen(PORT, () => {
  console.log(`Server is running natively at http://localhost:${PORT}`);
});
