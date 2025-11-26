/**
 * Skill Viewer Modal Component
 *
 * Displays detailed information about a loaded skill in a modal dialog.
 *
 * @module components/skills/SkillViewer
 */

import { useSkillStore } from '@/stores/skill.store'
import { Modal } from '@/components/ui/Modal'
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
      size="lg"
      variant="accent"
      title={skill?.name || ''}
    >
      {skill && (
        <div className="space-y-6 pt-4">
          {/* Description Section */}
          <div>
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
              Description
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">
              {skill.description}
            </p>
          </div>


          {/* Tags */}
          {skill.tags && skill.tags.length > 0 && (
            <div>
              <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
                <Tag className="w-4 h-4" />
                Tags
              </h3>
              <div className="flex flex-wrap gap-2">
                {skill.tags.map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center px-3 py-1 rounded-full text-sm bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-300 border border-purple-200 dark:border-purple-800"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Skill ID */}
          <div>
            <h3 className="text-xs font-semibold text-gray-500 dark:text-gray-400 mb-1 flex items-center gap-1">
              <Tag className="w-3.5 h-3.5" />
              Skill ID
            </h3>
            <code className="text-sm text-gray-800 dark:text-gray-200 bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
              {skill.id}
            </code>
          </div>
        </div>
      )}
    </Modal>
  )
}
