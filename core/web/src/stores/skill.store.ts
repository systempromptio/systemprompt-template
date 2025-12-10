import { create } from 'zustand'
import type { AgentSkill } from '@a2a-js/sdk'

interface SkillStore {
  byId: Record<string, AgentSkill>
  byContext: Readonly<Record<string, Readonly<Record<string, readonly string[]>>>>
  selectedSkillId: string | null

  addSkillToTask: (contextId: string, taskId: string, skill: AgentSkill) => void
  loadSkills: (skills: AgentSkill[]) => void
  clearContext: (contextId: string) => void
  openSkill: (skillId: string) => void
  closeSkill: () => void
  reset: () => void
}

export const useSkillStore = create<SkillStore>()((set) => ({
  byId: {},
  byContext: {},
  selectedSkillId: null,

  loadSkills: (skills) => {
    const byId: Record<string, AgentSkill> = {}
    skills.forEach(skill => {
      byId[skill.id] = skill
    })
    set({ byId })
  },

  addSkillToTask: (contextId, taskId, skill) => {
    set((state) => {
      const skillIds = state.byContext[contextId]?.[taskId]

      if (skillIds?.includes(skill.id)) {
        return state
      }

      const currentIds = skillIds || []
      const uniqueSkillIds = Array.from(new Set([...currentIds, skill.id]))

      const contextTasks = state.byContext[contextId] || {}
      const newContextTasks = { ...contextTasks, [taskId]: uniqueSkillIds }
      const newByContext = { ...state.byContext, [contextId]: newContextTasks }

      const existingSkill = state.byId[skill.id]
      const skillChanged =
        !existingSkill ||
        existingSkill.name !== skill.name ||
        existingSkill.description !== skill.description

      const newById = skillChanged ? { ...state.byId, [skill.id]: skill } : state.byId

      return {
        byId: newById,
        byContext: newByContext,
      }
    })
  },

  clearContext: (contextId) => {
    set((state) => {
      const newByContext = { ...state.byContext }
      delete newByContext[contextId]
      return { byContext: newByContext }
    })
  },

  openSkill: (skillId) => {
    set({ selectedSkillId: skillId })
  },

  closeSkill: () => {
    set({ selectedSkillId: null })
  },

  reset: () => {
    set({
      byId: {},
      byContext: {},
      selectedSkillId: null,
    })
  },
}))
