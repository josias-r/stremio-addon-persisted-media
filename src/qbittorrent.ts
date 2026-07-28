import { getReqEnvVariable } from "./loadenv.ts";

const QBITTORRENT_URL = getReqEnvVariable("QBITTORRENT_URL");
const QBITTORRENT_USERNAME = getReqEnvVariable("QBITTORRENT_USERNAME");
const QBITTORRENT_PASSWORD = getReqEnvVariable("QBITTORRENT_PASSWORD");

async function getSessionCookie(): Promise<string | null> {
  const loginBody = new URLSearchParams();
  loginBody.append("username", QBITTORRENT_USERNAME);
  loginBody.append("password", QBITTORRENT_PASSWORD);

  const loginRes = await fetch(`${QBITTORRENT_URL}/api/v2/auth/login`, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
      Origin: QBITTORRENT_URL,
      Referer: `${QBITTORRENT_URL}/`,
    },
    body: loginBody.toString(),
  });

  const loginText = await loginRes.text();
  if (!loginRes.ok || loginText === "Fails.") {
    console.error(`Failed to login to qBittorrent. Response: ${loginText}`);
    return null;
  }

  const cookieHeader = loginRes.headers.get("set-cookie");
  if (!cookieHeader) {
    console.error("No cookie returned from qBittorrent login");
    return null;
  }

  return cookieHeader.split(";")[0];
}

export async function addTorrentToQbit(magnetUri: string): Promise<boolean> {
  try {
    const sid = await getSessionCookie();
    if (!sid) return false;

    const addFormData = new FormData();
    addFormData.append("urls", magnetUri);

    const addRes = await fetch(`${QBITTORRENT_URL}/api/v2/torrents/add`, {
      method: "POST",
      headers: { Cookie: sid },
      body: addFormData,
    });

    if (!addRes.ok) {
      console.error(`Failed to add torrent to qBittorrent: ${addRes.status}`);
      return false;
    }

    return true;
  } catch (error) {
    console.error("Error communicating with qBittorrent:", error);
    return false;
  }
}

export interface QbitFile {
  name: string; // The file name/path relative to the torrent's root
  size: number;
  progress: number;
}

export async function getTorrentFiles(infoHash: string): Promise<QbitFile[]> {
  try {
    const sid = await getSessionCookie();
    if (!sid) return [];

    const filesRes = await fetch(
      `${QBITTORRENT_URL}/api/v2/torrents/files?hash=${infoHash}`,
      { headers: { Cookie: sid } },
    );

    if (!filesRes.ok) {
      console.error(
        `Failed to fetch files for ${infoHash}: ${filesRes.status}`,
      );
      return [];
    }

    return (await filesRes.json()) as QbitFile[];
  } catch (error) {
    console.error(`Error fetching files for ${infoHash}:`, error);
    return [];
  }
}
