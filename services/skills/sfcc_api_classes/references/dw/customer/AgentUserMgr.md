 # dw.customer.AgentUserMgr

 ## Overview
 Helper methods for agent user authentication and login-on-behalf functionality.

 ## Description
 Provides static helpers to log in/out agent users and to authenticate a specified customer into the current session when the agent has appropriate permissions. Methods return `Status` objects indicating success or failure.

 ```ts
 declare class AgentUserMgr  {
     /** Log in an agent user by credentials; returns Status. */
     static loginAgentUser(login: string, password: string): Status

     /** Log in a customer on behalf of an agent (requires permission); returns Status. */
     static loginOnBehalfOfCustomer(customer: Customer): Status

     /** Logout agent user and current customer; returns Status. */
     static logoutAgentUser(): Status
 }
 ```
