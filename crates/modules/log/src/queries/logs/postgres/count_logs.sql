SELECT COUNT(*) 
FROM logs 
WHERE 
 ($1 IS NULL OR level = $1) AND 
 ($2 IS NULL OR module = $2) AND 
 ($3 IS NULL OR message LIKE '%' || $3 || '%')