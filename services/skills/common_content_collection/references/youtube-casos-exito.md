# Casos de Éxito Foodles — YouTube

## Playlist Principal
- **URL:** https://www.youtube.com/playlist?list=PL9KT1AOP2NPTdVMjx813ZQGTVo8XiSWu5
- **Nombre:** Casos de éxito FOODLES
- **Canal:** https://www.youtube.com/@foodles_es

## Videos de la Playlist

| # | Empresa | Video ID | URL |
|---|---------|----------|-----|
| 1 | Espacio Orgánico | g73Rltg6pFY | https://www.youtube.com/watch?v=g73Rltg6pFY |
| 2 | Prima Protección | G4JSTQTZBhk | https://www.youtube.com/watch?v=G4JSTQTZBhk |
| 3 | Todotrofeo | rxxIy4MiHVE | https://www.youtube.com/watch?v=rxxIy4MiHVE |
| 4 | Kilnher | FG7pXnI59qY | https://www.youtube.com/watch?v=FG7pXnI59qY |
| - | Frooty | WVmC7I4nnu8 | https://www.youtube.com/watch?v=WVmC7I4nnu8 |
| - | Wiselegal | Kmek5oK1SZc | https://www.youtube.com/watch?v=Kmek5oK1SZc |
| - | Recolim | aHrG3X3LiFA | https://www.youtube.com/watch?v=aHrG3X3LiFA |

## YouTube API

API Key guardada en `/root/clawd/.env.odoo` como `YOUTUBE_API_KEY`

```bash
# Obtener título de un video
source /root/clawd/.env.odoo
curl -s "https://www.googleapis.com/youtube/v3/videos?part=snippet&id=VIDEO_ID&key=$YOUTUBE_API_KEY" | jq -r '.items[0].snippet.title'
```

## Uso en contenido

### Para emails (thumbnail + link)
```html
<a href="https://www.youtube.com/watch?v=WVmC7I4nnu8">
  <img src="https://img.youtube.com/vi/WVmC7I4nnu8/hqdefault.jpg" alt="Caso de éxito">
</a>
```

### Para blogs/web (embed)
```html
<iframe width="560" height="315" 
        src="https://www.youtube.com/embed/WVmC7I4nnu8" 
        frameborder="0" allowfullscreen></iframe>
```

### Para redes sociales
- Link directo al video
- Mencionar empresa y sector
- Hashtags: #OdooCasosDeÉxito #Foodles #Odoo

## Thumbnails

| Tamaño | URL Pattern |
|--------|-------------|
| 120x90 | `https://img.youtube.com/vi/VIDEO_ID/default.jpg` |
| 320x180 | `https://img.youtube.com/vi/VIDEO_ID/mqdefault.jpg` |
| 480x360 | `https://img.youtube.com/vi/VIDEO_ID/hqdefault.jpg` |
| 1280x720 | `https://img.youtube.com/vi/VIDEO_ID/maxresdefault.jpg` |
