export function frontmatterValue(body: string, key: string) {
  const lines = body.replace(/^\uFEFF/, '').trimStart().split(/\r?\n/)
  while (lines.length) {
    const first = lines[0]?.trim() ?? ''
    if (first && !first.startsWith('<!--')) break
    lines.shift()
  }

  if (lines.shift()?.trim() !== '---') return ''

  const pattern = new RegExp(`^${key}[:：]\\s*(.*)$`, 'i')
  for (const line of lines) {
    const trimmed = line.trim()
    if (trimmed === '---') break
    const match = trimmed.match(pattern)
    if (match) return match[1].trim().replace(/^['"]|['"]$/g, '')
  }

  return ''
}

export function markdownDescription(body: string) {
  return frontmatterValue(body, 'description')
}
