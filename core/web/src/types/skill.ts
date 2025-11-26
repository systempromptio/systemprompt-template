/**
 * Skill metadata types matching Rust backend structures
 * Core: core/crates/shared/models/src/a2a/artifact_metadata.rs
 */

export interface SkillMetadata {
  skill_id: string
  skill_name: string
  skill_version?: number
}

/**
 * Extract skill metadata from artifact metadata
 * Returns null if skill metadata is incomplete or missing
 */
export function extractSkillMetadata(metadata: Record<string, unknown>): SkillMetadata | null {
  const skill_id = metadata?.skill_id
  const skill_name = metadata?.skill_name
  const skill_version = metadata?.skill_version

  if (typeof skill_id !== 'string' || typeof skill_name !== 'string') {
    return null
  }

  return {
    skill_id,
    skill_name,
    skill_version: typeof skill_version === 'number' ? skill_version : undefined
  }
}
