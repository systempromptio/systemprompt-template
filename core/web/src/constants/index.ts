/**
 * Application Constants
 *
 * Centralized definitions for all magic strings, enums, and constant values.
 * This prevents duplication and makes constants discoverable.
 */

export {
  EventType,
  ExecutionStatus,
  type EventType as EventTypeValue,
  type ExecutionStatus as ExecutionStatusValue,
} from './events'

export {
  ArtifactType,
  ArtifactMetadataKey,
  ArtifactSource,
  type ArtifactType as ArtifactTypeValue,
  type ArtifactMetadataKey as ArtifactMetadataKeyValue,
  type ArtifactSource as ArtifactSourceValue,
} from './artifacts'

export {
  HTTP_STATUS,
  HTTP_HEADERS,
  CONTENT_TYPES,
  AUTH_SCHEME,
} from './http'

export {
  UIStateKey,
  Theme,
  DialogType,
  Animation,
  type UIStateKey as UIStateKeyValue,
  type Theme as ThemeValue,
  type DialogType as DialogTypeValue,
  type Animation as AnimationValue,
} from './ui'

export {
  stepTypeIcons,
  stepTypeLabels,
  getStepTypeIcon,
  getStepTypeLabel,
} from './stepTypeIcons'
