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

export async function fetchJackettResults(
  query: string,
): Promise<JackettResult[]> {
  const JACKETT_URL = process.env.JACKETT_URL;
  const JACKETT_API_KEY = process.env.JACKETT_API_KEY;

  if (!JACKETT_URL) {
    throw new Error("JACKETT_URL environment variable is required.");
  }
  if (!JACKETT_API_KEY) {
    throw new Error("JACKETT_API_KEY environment variable is required.");
  }

  const url = `${JACKETT_URL}/api/v2.0/indexers/all/results?apikey=${JACKETT_API_KEY}&Query=${encodeURIComponent(query)}`;

  try {
    const response = await fetch(url);
    if (!response.ok) {
      console.error(`Jackett API responded with status: ${response.status}`);
      return [];
    }

    const data = (await response.json()) as any;
    let results: JackettResult[] = data.Results || [];

    // Jackett's API does not universally support filtering/sorting by seeders directly in the query string,
    // because it acts as a proxy to various underlying trackers that have different native capabilities.
    // Therefore, we must filter out 0 seeders and sort the results here in Javascript.
    results = results.filter((r) => r.Seeders > 0);

    return results.sort((a, b) => (b.Seeders || 0) - (a.Seeders || 0));
  } catch (error) {
    console.error("Failed to fetch from Jackett API:", error);
    return [];
  }
}
