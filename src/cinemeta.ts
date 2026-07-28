export async function getCinemetaTitle(type: "movie" | "series", imdbId: string): Promise<string | null> {
  try {
    const url = `https://v3-cinemeta.strem.io/meta/${type}/${imdbId}.json`;
    const response = await fetch(url);
    if (!response.ok) {
      console.error(`Cinemeta API responded with status: ${response.status} for ${imdbId}`);
      return null;
    }
    const data = await response.json() as any;
    if (data && data.meta && data.meta.name) {
      return data.meta.name;
    }
    return null;
  } catch (error) {
    console.error(`Failed to fetch title from Cinemeta for ${imdbId}:`, error);
    return null;
  }
}
