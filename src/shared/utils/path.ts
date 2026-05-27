export function pathName(value: string) {
  return value.trim().replace(/[\\/]+$/, '').split(/[\\/]/).filter(Boolean).at(-1) ?? ''
}

export function repoName(value: string) {
  return pathName(value.trim().replace(/\.git$/i, '').replace(/[\\/]+$/, '').replace(/:/g, '/'))
}
