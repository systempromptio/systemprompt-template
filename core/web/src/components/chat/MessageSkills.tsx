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
    <div className={cn('flex flex-wrap gap-2 mt-xs', className)}>
      {skills.map((skill: AgentSkill) => (
        <button
          key={skill.id}
          onClick={() => openSkill(skill.id)}
          className="inline-flex items-center gap-xs px-md py-xs bg-primary/10 hover:bg-primary/20 border border-primary/20 rounded-lg text-sm text-primary transition-colors cursor-pointer"
          title={skill.description}
        >
          <BookOpen className="w-4 h-4" />
          <span>{skill.name}</span>
        </button>
      ))}
    </div>
  )
})
