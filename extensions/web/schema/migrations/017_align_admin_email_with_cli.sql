-- Align the bootstrap admin user's email with what the CLI's local-trial
-- session resolver looks up. Core's resolve_local_user_email() returns
-- "admin@localhost.dev" for local trial profiles; migration 016 set
-- "admin@localhost", causing find_by_email to miss and the resolver to
-- attempt a create that then collides on the name='admin' unique key.

UPDATE users
   SET email = 'admin@localhost.dev'
 WHERE name = 'admin'
   AND email = 'admin@localhost';
