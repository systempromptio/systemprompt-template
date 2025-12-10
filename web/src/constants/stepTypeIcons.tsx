import {
  Brain,
  CheckCircle,
  Map,
  Sparkles,
  Wrench,
} from 'lucide-react'
import type { StepType } from '@/types/execution'

export const stepTypeIcons: Record<StepType, React.ReactNode> = {
  understanding: <Brain className="w-4 h-4" />,
  planning: <Map className="w-4 h-4" />,
  skill_usage: <Sparkles className="w-4 h-4" />,
  tool_execution: <Wrench className="w-4 h-4" />,
  completion: <CheckCircle className="w-4 h-4" />,
}

export const stepTypeLabels: Record<StepType, string> = {
  understanding: 'Understanding',
  planning: 'Planning',
  skill_usage: 'Using Skill',
  tool_execution: 'Executing',
  completion: 'Complete',
}

export function getStepTypeIcon(stepType: StepType): React.ReactNode {
  return stepTypeIcons[stepType] || <Wrench className="w-4 h-4" />
}

export function getStepTypeLabel(stepType: StepType): string {
  return stepTypeLabels[stepType] || 'Processing'
}
