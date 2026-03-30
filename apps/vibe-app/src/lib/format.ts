export function formatDateTime(epochMs: number | null | undefined) {
  if (!epochMs) {
    return "—";
  }

  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  }).format(epochMs);
}

export function formatRelativeTime(epochMs: number | null | undefined) {
  if (!epochMs) {
    return "—";
  }

  const diffMs = epochMs - Date.now();
  const absMs = Math.abs(diffMs);
  const formatter = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;

  if (absMs < hour) {
    return formatter.format(Math.round(diffMs / minute), "minute");
  }
  if (absMs < day) {
    return formatter.format(Math.round(diffMs / hour), "hour");
  }
  return formatter.format(Math.round(diffMs / day), "day");
}

export function formatDuration(startedAtEpochMs: number | null, finishedAtEpochMs?: number | null) {
  if (!startedAtEpochMs) {
    return "—";
  }

  const end = finishedAtEpochMs ?? Date.now();
  const seconds = Math.max(0, Math.round((end - startedAtEpochMs) / 1000));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainSeconds = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${remainSeconds}s`;
  }
  return `${remainSeconds}s`;
}

export function formatCount(value: number) {
  return new Intl.NumberFormat().format(value);
}
