# dw.system.AgentUserStatusCodes

## Overview
**Deprecated** - Use dw.customer.AgentUserStatusCodes instead. Contains status code constants for agent user login.

## Description
AgentUserStatusCodes contains constants representing status codes that can be used with a Status object to indicate the success or failure of the agent user login process. This class should only be used for the LoginAgentUser / LoginOnBehalfCustomer pipelets.

```
Object
  dw.customer.AgentUserStatusCodes
    dw.system.AgentUserStatusCodes
```

```ts
/**
 * @deprecated Use dw.customer.AgentUserStatusCodes instead - this class should only be used for the LoginAgentUser / LoginOnBehalfCustomer pipelets
 */
declare class AgentUserStatusCodes extends dw.customer.AgentUserStatusCodes {
	/**
	 * Constructs an AgentUserStatusCodes instance.
	 * @deprecated Use dw.customer.AgentUserStatusCodes
	 */
	constructor()
}
```
