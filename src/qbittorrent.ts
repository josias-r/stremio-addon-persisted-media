import { getReqEnvVariable } from "./loadenv.ts";

const QBITTORRENT_URL = getReqEnvVariable("QBITTORRENT_URL");
const QBITTORRENT_USERNAME = getReqEnvVariable("QBITTORRENT_USERNAME");
const QBITTORRENT_PASSWORD = getReqEnvVariable("QBITTORRENT_PASSWORD");

export async function addTorrentToQbit(magnetUri: string): Promise<boolean> {
  try {
    // 1. Login to get Session ID cookie
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
      console.error(
        `Failed to login to qBittorrent. Check your username/password! Response: ${loginText}`,
      );
      return false;
    }

    const cookieHeader = loginRes.headers.get("set-cookie");
    if (!cookieHeader) {
      console.error("No cookie returned from qBittorrent login");
      return false;
    }

    // Extract the SID=... part of the cookie
    const sid = cookieHeader.split(";")[0];

    // 2. Add torrent
    const addFormData = new FormData();
    addFormData.append("urls", magnetUri);

    const addRes = await fetch(`${QBITTORRENT_URL}/api/v2/torrents/add`, {
      method: "POST",
      headers: {
        Cookie: sid,
      },
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
