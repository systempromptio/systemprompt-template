


```json
{
  "@type-id": "custom.MyCustomChunkScriptModuleStep",
  "@supports-parallel-execution": "false",
  "@supports-site-context": "true",
  "@supports-organization-context": "false",
  "description": "My custom chunk script step type",
  "module": "my_cartridge/cartridge/scripts/steps/myModule.js",
  "before-step-function": "beforeStep",
  "total-count-function": "getTotalCount",
  "before-chunk-function": "beforeChunk",
  "read-function": "read",
  "process-function": "process",
  "write-function": "write",
  "after-chunk-function": "afterChunk",
  "after-step-function": "afterStep",
  "chunk-size": 10,
  "transactional": "true",
  "parameters": {
    "parameter": []
  },
  "status-codes": {
    "status": []
  }
}
```


- `@type-id` - Required -	Identifies the step type. 
This is the name that users see in Business Manager, so make it descriptive. Must begin with custom..
Must not contain leading or trailing white space or more than 100 characters.
Must be unique within the job definition. You can't register multiple steps with the same @type-id in different cartridges.
The @type-id is validated as unique by parsing the steptypes files from all cartridges on the cartridge path. If there is a step with the same @type-id, the step isn't loaded.
The @type-id value can't be the same as any system step, for example, ExecutePipeline or IncludeStepsFromJob.

- `@supports-parallel-execution`	Optional -	Determines if the step type can be used in parallel with itself or other step types.
Must have value true or false. Default is true.
If false, split flows that contain steps of this type are always executed sequentially and never in parallel.
If true, split flows that contain steps of this type are executed in parallel, as long as:
  - The split flows don't contain steps with supports-parallel-execution = false.
  - The split flows are not configured to be executed sequentially.
  - There are enough resources available to do parallel execution.

- `@supports-site-context` - Optional -Determines if the step type is intended to be used in site context.
Must have value true or false. Default is true.
If true, steps of this type can be used for flows with one or more sites as scope.
If false, steps of this type can't be used for flows with one or more sites as scope.
@supports-organization-context and @supports-site-context cannot both have the same true or false setting. If @supports-site-context is false, @supports-organization-context must be true and vice versa.
- `@supports-organization-context` -	Optional - Determines if the step type is intended to be used in organization context.
Must have value true or false. Default is true.
If true, steps of this type can be used for flows with organization scope.
If false, steps of this type can't be used for flows with organization scope.
@supports-organization-context and @supports-site-context cannot both have the same true or false setting. If @supports-site-context is false, @supports-organization-context must be true and vice versa.
- `description`	- Optional - The description of the step type. Not shown in Business Manager. Must not exceed 4000 characters.
- `module` - Required -	Path to the script module to be executed. Must not contain leading or trailing white space.
- `before-step-function` - Optional - References the function of the chunk-oriented script module to be executed before the step begins.
If not defined, no before-step logic is used.
- `total-count-function` - Optional - References the function to be executed to determine the total number of items.
If not defined, no total count is used.
- `before-chunk-function` - Optional - References the function of the chunk-oriented script module to be executed before a chunk begins.
If not defined, no before-chunk logic is used.
- `read-function` - Optional - Read function of the chunk script.
If not defined, script must export a function named read.
- `process-function` - Optional - Process function of the chunk script.
If not defined, script must export a function named process.
- `write-function` - Optional - References the write function of the chunk script.
If not defined, script must export a function named write.
- `after-chunk-function` - Optional - References the function of the chunk-oriented script module to be executed after a chunk has finished.
If not defined, no after-chunk logic is used.
- `after-step-function` - Optional - References the function of the chunk-oriented script module to be executed after the step has finished.
If not defined, no after-step logic is used.
- `chunk-size` - Required - Defines the size of one chunk. Must be a numeric value greater than zero. Must not contain leading or trailing white space.
- `transactional` - Optional - Indicates if the module requires a database transaction.
Must have value true or false. Default is false.
If this value is set to true, the job step executes as a single, potentially very large, transaction.
To avoid a negative impact on system performance and allow more granular transaction control, keep the default setting of false. Implement transaction handling within the job step using the dw.system.Transaction API.
- `parameters` - Optional - Parameters for the step, which the user configures in Business Manager. Contains one parameter array-element with one or more parameter objects.
- `status-codes` - Optional - Defines the meta data for status codes returned for the step. Contains one status array-element with one or more status objects.
