# dw.system.Pipeline

## Overview
Executes pipelines from JavaScript controllers with local pipeline dictionary scope.

## Description
Invokes process pipelines synchronously within current request (similar to Call node). Best for pipelines ending with End nodes that don't span requests. Pipelines with Interaction-Continue nodes not supported. Called pipeline receives isolated dictionary; populate via args object, read results from returned PipelineDictionary. Error branches invoked on exception; unhandled exceptions propagate to script. End node name available as 'EndNodeName' in returned dictionary. Requires API version >=15.5.

```ts
declare class Pipeline  {
	/**
	 * Executes pipeline with empty initial dictionary
	 * @param pipeline - Pipeline identifier format: 'PipelineName-StartNodeName'
	 * @returns Pipeline dictionary with results
	 */
	static execute(pipeline: string): PipelineDictionary

	/**
	 * Executes pipeline with initial dictionary values from args object properties
	 * @param pipeline - Pipeline identifier format: 'PipelineName-StartNodeName'
	 * @param args - Object whose properties initialize pipeline dictionary
	 * @returns Pipeline dictionary with results
	 */
	static execute(pipeline: string, args: Object): PipelineDictionary
}
```
