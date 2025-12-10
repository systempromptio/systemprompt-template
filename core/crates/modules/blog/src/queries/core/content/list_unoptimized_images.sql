SELECT id, slug, image
FROM markdown_content
WHERE image IS NOT NULL
  AND image != ''
  AND image NOT LIKE '%.webp'
  AND (
    image LIKE '%.png' OR
    image LIKE '%.jpg' OR
    image LIKE '%.jpeg' OR
    image LIKE '%.PNG' OR
    image LIKE '%.JPG' OR
    image LIKE '%.JPEG'
  )
ORDER BY updated_at DESC;
