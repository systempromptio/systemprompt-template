/**
 * Artifact Skills Extraction Utility
 *
 * Links artifacts to their skills in the task context.
 * Skills are pre-loaded from the database, this function only creates the association.
 *
 * Core: core/crates/shared/models/src/a2a/artifact_metadata.rs
 */

import type { Artifact } from '@/types/artifact'
import { extractSkillMetadata } from '@/types/skill'
import { useSkillStore } from '@/stores/skill.store'
import { logger } from '@/lib/logger'

/**
 * Extract skill metadata from artifact and link to task
 *
 * This function is idempotent - safe to call multiple times on the same artifact.
 * The skill store has built-in duplicate prevention.
 *
 * @param artifact - The artifact to extract skills from
 * @param contextId - The context ID this artifact belongs to
 * @param taskId - The task ID this artifact was created by
 */
export function extractAndStoreSkill(
  artifact: Artifact,
  contextId: string,
  taskId: string
): void {
  console.log('[SKILL EXTRACTION DEBUG] extractAndStoreSkill called:', {
    artifactId: artifact.artifactId,
    contextId,
    taskId,
    metadata: artifact.metadata
  })

  if (!contextId || !taskId) {
    logger.debug(
      'Skipping skill extraction - missing contextId or taskId',
      { artifactId: artifact.artifactId, contextId, taskId },
      'extractArtifactSkills'
    )
    return
  }

  const metadata = artifact.metadata as Record<string, unknown>
  const skillMetadata = extractSkillMetadata(metadata)

  if (!skillMetadata) {
    console.log('[SKILL EXTRACTION DEBUG] No skill metadata found')
    return
  }

  // Lookup skill from store (must be pre-loaded from API)
  const skill = useSkillStore.getState().byId[skillMetadata.skill_id]

  if (!skill) {
    logger.warn(
      'Skill not found in store',
      { skillId: skillMetadata.skill_id, artifactId: artifact.artifactId },
      'extractArtifactSkills'
    )
    return
  }

  console.log('[SKILL EXTRACTION DEBUG] Found skill in store:', skill)

  logger.debug(
    'Linking artifact to skill',
    {
      artifactId: artifact.artifactId,
      skillId: skill.id,
      skillName: skill.name,
      contextId,
      taskId,
    },
    'extractArtifactSkills'
  )

  useSkillStore.getState().addSkillToTask(contextId, taskId, skill)
  console.log('[SKILL EXTRACTION DEBUG] Linked skill to task')
}
