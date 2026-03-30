export function suppressProjectKey(hiddenProjectKeys: string[], projectKey: string) {
  return hiddenProjectKeys.includes(projectKey)
    ? hiddenProjectKeys
    : [...hiddenProjectKeys, projectKey];
}

export function removeSuppressedProjectKey(hiddenProjectKeys: string[], projectKey: string) {
  return hiddenProjectKeys.filter((entry) => entry !== projectKey);
}

export function clearRediscoveredHiddenProjectKeys(
  hiddenProjectKeys: string[],
  discoveredProjectKeys: Iterable<string>
) {
  const rediscovered = new Set(discoveredProjectKeys);
  return hiddenProjectKeys.filter((key) => !rediscovered.has(key));
}

export function filterVisibleProjectKeys<T extends { key: string }>(
  projects: T[],
  hiddenProjectKeys: string[]
) {
  if (!hiddenProjectKeys.length) {
    return projects;
  }

  const hidden = new Set(hiddenProjectKeys);
  return projects.filter((project) => !hidden.has(project.key));
}
