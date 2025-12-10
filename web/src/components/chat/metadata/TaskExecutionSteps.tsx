import React, { useState, useMemo } from 'react'
import { Zap } from 'lucide-react'
import { useUIStateStore } from '@/stores/ui-state.store'
import { ExecutionStepsModal } from '../ExecutionStepsModal'
import type { ExecutionStep } from '@/types/execution'

interface TaskExecutionStepsProps {
  taskId: string
}

export const TaskExecutionSteps = React.memo(function TaskExecutionSteps({
  taskId,
}: TaskExecutionStepsProps) {
  const [showModal, setShowModal] = useState(false)
  const stepsById = useUIStateStore((s) => s.stepsById)
  const stepIdsByTask = useUIStateStore((s) => s.stepIdsByTask)

  const steps = useMemo(() => {
    const stepIds = stepIdsByTask[taskId] || []
    return stepIds
      .map((id) => stepsById[id])
      .filter((step): step is ExecutionStep => !!step)
      .sort((a, b) => new Date(a.startedAt).getTime() - new Date(b.startedAt).getTime())
  }, [stepsById, stepIdsByTask, taskId])

  if (steps.length === 0) return null

  return (
    <>
      <span className="text-text-secondary/40">·</span>
      <button
        onClick={() => setShowModal(true)}
        className="inline-flex items-center gap-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
      >
        <Zap className="w-3 h-3" />
        <span>
          {steps.length} step{steps.length !== 1 ? 's' : ''}
        </span>
      </button>

      <ExecutionStepsModal
        isOpen={showModal}
        onClose={() => setShowModal(false)}
        steps={steps}
      />
    </>
  )
})
