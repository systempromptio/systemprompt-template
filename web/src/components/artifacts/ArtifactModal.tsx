import { Modal } from '@/components/ui/Modal'
import { ArtifactViewer } from '@/components/artifacts/ArtifactViewer'
import { useArtifactStore } from '@/stores/artifact.store'

export function ArtifactModal() {
  const selectedArtifactId = useArtifactStore((state) => state.selectedArtifactId)
  const ephemeralArtifact = useArtifactStore((state) => state.ephemeralArtifact)
  const byId = useArtifactStore((state) => state.byId)
  const closeArtifact = useArtifactStore((state) => state.closeArtifact)

  const artifact = ephemeralArtifact || (selectedArtifactId ? byId[selectedArtifactId] : null)
  const isOpen = artifact !== null && artifact !== undefined

  const handleClose = () => {
    closeArtifact()
  }

  if (!artifact) {
    return null
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      size="xl"
      variant="accent"
      showCloseButton={false}
      closeOnBackdrop={true}
      closeOnEscape={true}
      className="!max-w-[calc(100vw-2rem)] !max-h-[calc(100vh-2rem)] !w-[calc(100vw-2rem)] !h-[calc(100vh-2rem)]"
    >
      <ArtifactViewer artifact={artifact} onClose={handleClose} />
    </Modal>
  )
}
