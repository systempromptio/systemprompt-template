UPDATE logs 
SET level = $1, module = $2, message = $3, metadata = $4 
WHERE id = $5