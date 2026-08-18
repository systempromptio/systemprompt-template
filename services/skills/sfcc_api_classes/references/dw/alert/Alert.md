# dw.alert.Alert

## Overview
Represents a single system alert to be shown to a Business Manager user.

## Description
This class models a system alert, including its priority, display message, and context. Alerts are used to notify Business Manager users about important information or required actions. Each alert references a descriptor, may have a context object, and provides a remediation URL for resolution.

```ts
declare class Alert  {
  /**
   * String constant to denote the 'action required' priority.
   */
  static PRIORITY_ACTION: 'ACTION';

  /**
   * String constant to denote the 'informational' priority.
   */
  static PRIORITY_INFO: 'INFO';

  /**
   * String constant to denote the 'warning' priority.
   */
  static PRIORITY_WARN: 'WARN';

  /**
   * The ID of the referenced alert description.
   * @readonly
   */
  readonly alertDescriptorID: string;

  /**
   * The ID of the referenced context object (or null if not assigned).
   * @readonly
   */
  readonly contextObjectID: string;

  /**
   * Resolves the display message to be shown, with parameters replaced as needed.
   * @readonly
   */
  readonly displayMessage: string;

  /**
   * The priority assigned to the message. One of PRIORITY_INFO, PRIORITY_WARN, PRIORITY_ACTION.
   * @readonly
   */
  readonly priority: string;

  /**
   * The URL of the page where the user can resolve the alert.
   * @readonly
   */
  readonly remediationURL: string;

  /**
   * Returns the ID of the referenced alert description.
   */
  getAlertDescriptorID(): string;

  /**
   * Returns the ID of the referenced context object (or null if not assigned).
   */
  getContextObjectID(): string;

  /**
   * Resolves the display message to be shown.
   */
  getDisplayMessage(): string;

  /**
   * Returns the priority assigned to the message.
   */
  getPriority(): string;

  /**
   * Returns the remediation URL for the alert.
   */
  getRemediationURL(): string;
}
```
