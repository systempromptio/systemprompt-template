# dw.job.JobStepExecution

## Overview
Represents execution of a single step within a job and exposes step identifiers and parameters.

## Description
Provides access to step-level identifiers, the parent job execution, and parameter values defined for the step.

```ts
declare class JobStepExecution  {
    /** Read-only ID of this step execution. */
    ID: string

    /** Read-only JobExecution this step belongs to. */
    jobExecution: JobExecution

    /** Read-only ID of the step. */
    stepID: string

    /** Read-only ID of the step type. */
    stepTypeID: string

    /** Returns the ID of this step execution. */
    getID(): string

    /** Returns the JobExecution this step belongs to. */
    getJobExecution(): JobExecution

    /** Returns the value of a named parameter for the step. */
    getParameterValue(name: string): Object

    /** Returns the step ID. */
    getStepID(): string

    /** Returns the step type ID. */
    getStepTypeID(): string
}
```
