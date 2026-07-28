// A conservative filter that returns true ONLY if we are SURE it is the wrong episode.
// If it might be a season pack, or we can't parse it, we return false to keep it.
export function isDefinitelyWrongEpisode(
  title: string,
  expectedSeason: number,
  expectedEpisode: number,
): boolean {
  // Normalize string to avoid case issues
  const str = title.toUpperCase();

  // Pattern 1: S01E05 or S01E05-E06 or S01E05-06
  const sxxExxMatch = str.match(/S(\d+)E(\d+)(?:(?:-E|-)(\d+))?/);
  if (sxxExxMatch) {
    const s = parseInt(sxxExxMatch[1], 10);
    const eStart = parseInt(sxxExxMatch[2], 10);
    const eEnd = sxxExxMatch[3] ? parseInt(sxxExxMatch[3], 10) : eStart;

    if (s === expectedSeason) {
      if (expectedEpisode >= eStart && expectedEpisode <= eEnd) {
        return false; // It's our episode
      }
      return true; // Right season, WRONG episode
    } else {
      return true; // Wrong season entirely
    }
  }

  // Pattern 2: 1x05 or 1x05-06
  // Prevent matching codecs like 5.1.X264 by checking word boundaries strictly
  const axBMatch = str.match(
    /(?:^|[\s_\[-])(\d+)X(\d+)(?:-(\d+))?(?:[\s_\]-]|$)/,
  );
  if (axBMatch) {
    const s = parseInt(axBMatch[1], 10);
    const eStart = parseInt(axBMatch[2], 10);
    const eEnd = axBMatch[3] ? parseInt(axBMatch[3], 10) : eStart;

    if (s === expectedSeason) {
      if (expectedEpisode >= eStart && expectedEpisode <= eEnd) {
        return false;
      }
      return true;
    } else {
      // If s > 30, it's likely a resolution (e.g. 1920x1080), so ignore this match.
      if (s < 30) {
        return true;
      }
    }
  }

  // Pattern 3: Absolute episode numbers like E05 or EP05 or Episode 5
  // We only trust this if it's explicitly labeled "E", "EP", "EPISODE"
  const epMatch = str.match(
    /(?:^|[^A-Z])(?:E|EP|EPISODE)[.\s-]*(\d+)(?:(?:-E?|-)(\d+))?(?:\b|$)/,
  );
  if (epMatch) {
    const eStart = parseInt(epMatch[1], 10);
    const eEnd = epMatch[2] ? parseInt(epMatch[2], 10) : eStart;

    // If it explicitly mentions our episode in the range, keep it.
    if (expectedEpisode >= eStart && expectedEpisode <= eEnd) {
      return false;
    }

    return true;
  }

  // If no patterns match, we are not sure, so we DO NOT filter it.
  return false;
}

// Returns true if the title explicitly mentions exactly ONE episode (e.g. S01E05)
// OR if it explicitly mentions a season that does NOT include the expected season.
// If it's a season pack for our season (S01 or S01E01-E10 or S01-S03), it returns false.
export function isDefinitelyNotFullSeason(
  title: string,
  expectedSeason: number,
): boolean {
  const str = title.toUpperCase();

  const sxxExxMatch = str.match(/S(\d+)E(\d+)(?:(?:-E|-)(\d+))?/);
  if (sxxExxMatch) {
    const s = parseInt(sxxExxMatch[1], 10);
    if (s !== expectedSeason) return true; // Wrong season

    const eStart = parseInt(sxxExxMatch[2], 10);
    const eEnd = sxxExxMatch[3] ? parseInt(sxxExxMatch[3], 10) : eStart;
    if (eStart === eEnd) return true; // Exactly one episode
  }

  const axBMatch = str.match(
    /(?:^|[\s_\[-])(\d+)X(\d+)(?:-(\d+))?(?:[\s_\]-]|$)/,
  );
  if (axBMatch) {
    const s = parseInt(axBMatch[1], 10);
    if (s < 30) {
      if (s !== expectedSeason) return true; // Wrong season

      const eStart = parseInt(axBMatch[2], 10);
      const eEnd = axBMatch[3] ? parseInt(axBMatch[3], 10) : eStart;
      if (eStart === eEnd) return true; // Exactly one episode
    }
  }

  const epMatch = str.match(
    /(?:^|[^A-Z])(?:E|EP|EPISODE)[.\s-]*(\d+)(?:(?:-E?|-)(\d+))?(?:\b|$)/,
  );
  if (epMatch) {
    const eStart = parseInt(epMatch[1], 10);
    const eEnd = epMatch[2] ? parseInt(epMatch[2], 10) : eStart;
    if (eStart === eEnd) return true;
  }

  // Check for explicit season mentions (e.g. S02 or S01-S03 or Season 2)
  const seasonMatch = str.match(
    /(?:^|[^A-Z])(?:S|SEASON)[.\s-]*(\d+)(?:(?:-S?|-)(\d+))?(?:\b|$)/,
  );
  if (seasonMatch) {
    const sStart = parseInt(seasonMatch[1], 10);
    const sEnd = seasonMatch[2] ? parseInt(seasonMatch[2], 10) : sStart;

    if (expectedSeason < sStart || expectedSeason > sEnd) {
      return true; // Mentions seasons, but doesn't include our expected season
    }
  }

  return false;
}
