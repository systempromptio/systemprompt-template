# dw.system.Transaction

## Overview
Provides transactional context for atomic changes to persistent business objects with begin-commit-rollback operations.

## Description
Represents the current transaction providing context for atomic changes to persistent business objects. Must start transaction via `begin()` before creating, changing, or deleting business objects. Changes are durable only when committed via `commit()`. Rollback reverts all changes to previous state. Supports nested transactions (begin-begin-commit-commit requires symmetrical commits) and sequential transactions (begin-commit-begin-commit). Exception during transaction prevents commit, only allows rollback. If transaction open at end of pipeline/controller/job step, remaining changes committed unless exception thrown.

Best practices: avoid long running transactions in jobs; use one transaction for related changes needing joint rollback; don't begin/commit huge number of small transactions in loop; avoid changing same objects in parallel transactions.

```ts
declare class Transaction  {
	/**
	 * Begins a transaction.
	 */
	static begin(): void

	/**
	 * Commits the current transaction. Must have been started with begin() before.
	 */
	static commit(): void

	/**
	 * Rolls back the current transaction. Must have been started with begin() before.
	 */
	static rollback(): void

	/**
	 * Encloses the provided callback function in begin-commit transactional context. If transaction cannot be committed successfully, it's rolled back and exception is thrown.
	 * @param callback - Function to execute within transactional context.
	 * @returns Result of the callback function, if it returns something.
	 */
	static wrap(callback: Function): Object
}
```
