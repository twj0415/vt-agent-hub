import type { ProjectOutputScan } from '@/shared/api/client'
import type { BadgeTone } from '@/shared/types/ui'

export type ProjectStatusTone = Extract<BadgeTone, 'ready' | 'error' | 'warning'>

export type ProjectStatusMeta = {
  labelKey: string
  noteKey: string
  tone: ProjectStatusTone
}

const normalMeta: ProjectStatusMeta = {
  labelKey: 'common.normal',
  noteKey: 'pages.projects.notes.normal',
  tone: 'ready',
}

const abnormalMeta: ProjectStatusMeta = {
  labelKey: 'common.abnormal',
  noteKey: 'pages.projects.notes.pathMissing',
  tone: 'error',
}

// 项目目录正常但 AGENTS.md 被用户在 OS 层面删了——需要明显红色警告，否则用户看不出失同步。
const targetMissingMeta: ProjectStatusMeta = {
  labelKey: 'pages.projects.notes.targetMissingShort',
  noteKey: 'pages.projects.notes.targetMissing',
  tone: 'error',
}

const needsRepairMeta: ProjectStatusMeta = {
  labelKey: 'pages.projects.card.needsRepair',
  noteKey: 'pages.projects.notes.needsRepair',
  tone: 'warning',
}

export function projectStatusMeta(scan: ProjectOutputScan | null): ProjectStatusMeta {
  if (!scan) return normalMeta
  if (scan.issues.includes('project_path_missing')) return abnormalMeta
  // managed=true 但目标不存在 = 用户从文件系统层面删除了已接管的 AGENTS.md
  if (scan.managed && !scan.targetExists) return targetMissingMeta
  if (scan.issues.includes('unmanaged_existing')) return needsRepairMeta
  return normalMeta
}

export function projectStatusNoteKey(scan: ProjectOutputScan | null): string {
  if (!scan) return 'pages.projects.notes.scanNeeded'
  if (scan.issues.includes('project_path_missing')) return 'pages.projects.notes.pathMissing'
  if (scan.managed && !scan.targetExists) return 'pages.projects.notes.targetMissing'
  if (scan.issues.includes('unmanaged_existing')) return 'pages.projects.notes.needsRepair'
  if (scan.issues.includes('missing_target')) return 'pages.projects.notes.notApplied'
  if (scan.issues.includes('no_bound_rules')) return 'pages.projects.notes.bindRules'
  return 'pages.projects.notes.normal'
}
