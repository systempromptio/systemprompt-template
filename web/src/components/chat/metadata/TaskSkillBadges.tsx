import React from 'react'
import { useShallow } from 'zustand/react/shallow'
import { BookOpen } from 'lucide-react'
import { useSkillStore } from '@/stores/skill.store'
import type { AgentSkill } from '@a2a-js/sdk'

interface TaskSkillBadgesProps {
  taskId: string
  contextId: string
}

function useTaskSkills(taskId: string, contextId: string): readonly AgentSkill[] {
  return useSkillStore(
    useShallow((state): AgentSkill[] => {
      const contextTasks = state.byContext[contextId]
      if (!contextTasks) return []

      const taskSkillIds = contextTasks[taskId]
      if (!taskSkillIds) return []

      const skillsMap = new Map<string, AgentSkill>()
      taskSkillIds.forEach((id: string) => {
        const skill = state.byId[id]
        if (skill) skillsMap.set(skill.id, skill)
      })

      return Array.from(skillsMap.values())
    })
  )
}

export const TaskSkillBadges = React.memo(function TaskSkillBadges({
  taskId,
  contextId,
}: TaskSkillBadgesProps) {
  const skills = useTaskSkills(taskId, contextId)
  const openSkill = useSkillStore((state) => state.openSkill)

  if (skills.length === 0) return null

  return (
    <>
      {skills.map((skill) => (
        <button
          key={skill.id}
          onClick={() => openSkill(skill.id)}
          className="inline-flex items-center gap-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
          title={skill.description}
        >
          <BookOpen className="w-3 h-3" />
          <span>{skill.name}</span>
        </button>
      ))}
      <span className="text-text-secondary/40">·</span>
    </>
  )
})
