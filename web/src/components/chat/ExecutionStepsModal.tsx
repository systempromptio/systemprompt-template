/**
 * ExecutionStepsModal component.
 *
 * Displays execution steps in a modal dialog.
 *
 * @module components/chat/ExecutionStepsModal
 */

import { Modal, ModalBody } from '@/components/ui/Modal'
import { ExecutionTimeline } from './ExecutionTimeline'
import type { ExecutionStep } from '@/types/execution'

interface ExecutionStepsModalProps {
  isOpen: boolean
  onClose: () => void
  steps: ExecutionStep[]
}

export function ExecutionStepsModal({ isOpen, onClose, steps }: ExecutionStepsModalProps) {
  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Execution Steps"
      size="md"
      variant="accent"
      closeOnBackdrop={true}
      closeOnEscape={true}
    >
      <ModalBody>
        <ExecutionTimeline mode="modal" steps={steps} />
      </ModalBody>
    </Modal>
  )
}
