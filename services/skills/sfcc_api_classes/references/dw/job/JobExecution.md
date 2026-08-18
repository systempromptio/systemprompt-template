# dw.job.JobExecution

## Overview
Represents an execution instance of a job and exposes its context and identifiers.

## Description
Allows access to job-scoped context data shared between steps and provides job execution identifiers.

```ts
declare class JobExecution  {
    /** Read-only map used as job context to share simple data between steps. */
    context: Map

    /** Read-only ID of this job execution. */
    ID: string

    /** Read-only ID of the job that this execution belongs to. */
    jobID: string

    /** Returns the job context Map. */
    getContext(): Map

    /** Returns the ID of this job execution. */
    getID(): string

    /** Returns the job ID this execution belongs to. */
    getJobID(): string
}
```
