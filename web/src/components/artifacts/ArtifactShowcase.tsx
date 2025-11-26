import { useState } from 'react'
import { ArtifactViewer } from './ArtifactViewer'
import { mockArtifacts } from '@/__tests__/fixtures/mockData'

export function ArtifactShowcase() {
  const [selectedType, setSelectedType] = useState<keyof typeof mockArtifacts>('table')

  const artifact = mockArtifacts[selectedType]

  return (
    <div className="p-8 max-w-6xl mx-auto">
      <h1 className="text-3xl font-bold mb-4">Artifact Renderer Showcase</h1>
      <p className="text-gray-600 mb-8">
        Demonstrating all artifact types with mock data from MCP tools
      </p>

      <div className="flex gap-2 mb-6 flex-wrap">
        {(Object.keys(mockArtifacts) as Array<keyof typeof mockArtifacts>).map((type) => (
          <button
            key={type}
            onClick={() => setSelectedType(type)}
            className={`px-4 py-2 rounded font-medium capitalize ${
              selectedType === type
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
          >
            {type}
          </button>
        ))}
      </div>

      <ArtifactViewer artifact={artifact} />

      <div className="mt-8 p-4 bg-gray-50 rounded-lg">
        <h3 className="font-semibold mb-2">Mock Data Structure</h3>
        <pre className="text-xs overflow-x-auto">
          {JSON.stringify(artifact, null, 2)}
        </pre>
      </div>
    </div>
  )
}
