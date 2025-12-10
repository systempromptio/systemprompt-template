/**
 * Skill Viewer Modal Component
 *
 * Displays detailed information about a loaded skill in a modal dialog.
 *
 * @module components/skills/SkillViewer
 */

import { useSkillStore } from '@/stores/skill.store'
import { Modal, ModalBody } from '@/components/ui/Modal'
import { Tag } from 'lucide-react'

/**
 * Modal component for viewing skill details.
 * Automatically shows the currently selected skill from the store.
 */
export function SkillViewer() {
  const selectedSkillId = useSkillStore((state) => state.selectedSkillId)
  const closeSkill = useSkillStore((state) => state.closeSkill)
  const skill = useSkillStore((state) =>
    selectedSkillId ? state.byId[selectedSkillId] : null
  )

  return (
    <Modal
      isOpen={!!(skill && selectedSkillId)}
      onClose={closeSkill}
      size="md"
      variant="default"
      title={skill?.name || ''}
    >
      {skill && (
        <ModalBody>
          <div className="space-y-lg">
            {/* Description */}
            <div>
              <h3 className="text-sm font-semibold text-text-primary mb-sm">
                Description
              </h3>
              <p className="text-sm text-text-secondary leading-relaxed">
                {skill.description}
              </p>
            </div>

            {/* Tags */}
            {skill.tags && skill.tags.length > 0 && (
              <div>
                <h3 className="text-sm font-semibold text-text-primary mb-sm flex items-center gap-2">
                  <Tag className="w-4 h-4" />
                  Tags
                </h3>
                <div className="flex flex-wrap gap-2">
                  {skill.tags.map((tag) => (
                    <span
                      key={tag}
                      className="inline-flex items-center px-md py-xs rounded-lg text-sm bg-primary/10 text-primary border border-primary/20"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Skill ID */}
            <div>
              <h3 className="text-xs font-semibold text-text-secondary mb-xs flex items-center gap-1">
                <Tag className="w-3.5 h-3.5" />
                Skill ID
              </h3>
              <code className="text-sm text-text-primary bg-surface-variant px-2 py-1 rounded">
                {skill.id}
              </code>
            </div>
          </div>
        </ModalBody>
      )}
    </Modal>
  )
}
