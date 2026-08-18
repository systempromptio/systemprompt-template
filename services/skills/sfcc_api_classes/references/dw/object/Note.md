# dw.object.Note

## Overview
Represents a note attachable to persistent objects that support notes.

## Description
Contains read-only properties for creator, creation date, subject, and text of the note.

```ts
declare class Note  {
    /** Login ID of the user that created the note (read-only). */
    createdBy: string

    /** Date and time the note was created (read-only). */
    creationDate: Date

    /** Subject of the note (read-only). */
    subject: string

    /** Text of the note (read-only). */
    text: string

    /** Returns the username of the creator. */
    getCreatedBy(): string

    /** Returns the creation date. */
    getCreationDate(): Date

    /** Returns the subject. */
    getSubject(): string

    /** Returns the text. */
    getText(): string
}
```
