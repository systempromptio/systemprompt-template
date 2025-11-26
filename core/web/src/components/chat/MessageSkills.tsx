import React from 'react'
import { useShallow } from 'zustand/react/shallow'
import { useSkillStore } from '@/stores/skill.store'
import type { AgentSkill } from '@a2a-js/sdk'
import { BookOpen } from 'lucide-react'
import { cn } from '@/lib/utils/cn'

interface MessageSkillsProps {
  taskId?: string
  contextId: string
  className?: string
}

export const MessageSkills = React.memo(function MessageSkills({
  taskId,
  contextId,
  className,
}: MessageSkillsProps) {
  const skills = useSkillStore(
    useShallow((state): AgentSkill[] => {
      if (!taskId) return []

      const contextTasks = state.byContext[contextId]
      if (!contextTasks) return []

      const taskSkillIds = contextTasks[taskId]
      if (!taskSkillIds) return []

      // Map to skills and deduplicate by id as a safety net
      const skillsMap = new Map<string, AgentSkill>()
      taskSkillIds.forEach((id: string) => {
        const skill = state.byId[id]
        if (skill) {
          skillsMap.set(skill.id, skill)
        }
      })

      return Array.from(skillsMap.values())
    })
  )
  const openSkill = useSkillStore((state) => state.openSkill)

  if (!taskId || skills.length === 0) {
    return null
  }

  return (
    <div className={cn('flex flex-wrap gap-2 mt-2', className)}>
      {skills.map((skill: AgentSkill) => (
        <button
          key={skill.id}
          onClick={() => openSkill(skill.id)}
          className={cn(
            'inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-sm',
            'bg-purple-50 text-purple-700 hover:bg-purple-100',
            'dark:bg-purple-900/30 dark:text-purple-300 dark:hover:bg-purple-900/50',
            'transition-colors duration-200',
            'border border-purple-200 dark:border-purple-800'
          )}
          title={skill.description}
        >
          <BookOpen className="w-3.5 h-3.5" />
          <span className="font-medium">{skill.name}</span>
        </button>
      ))}
    </div>
  )
})
