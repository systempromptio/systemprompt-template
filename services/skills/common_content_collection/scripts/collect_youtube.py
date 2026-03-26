#!/usr/bin/env python3
"""
Collect video metadata and thumbnails from the Indaws YouTube channel.

Requires:
    - YOUTUBE_API_KEY environment variable
    - google-api-python-client package: pip install google-api-python-client

Usage:
    python collect_youtube.py
"""

import json
import os
import sys
import urllib.request
from datetime import datetime
from pathlib import Path

try:
    from dotenv import load_dotenv
except ImportError:
    load_dotenv = None

SCRIPT_DIR = Path(__file__).parent
SKILL_DIR = SCRIPT_DIR.parent
DATA_DIR = SKILL_DIR / "data"
THUMBNAILS_DIR = SKILL_DIR / "assets" / "thumbnails"

CHANNEL_HANDLE = "@indaws-odoo"


def load_env():
    """Load environment variables from .env file."""
    env_path = SKILL_DIR / ".env"
    if env_path.exists() and load_dotenv:
        load_dotenv(env_path)
    elif env_path.exists():
        # Fallback: manually parse .env file
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, value = line.split("=", 1)
                    value = value.strip().strip('"').strip("'")
                    os.environ[key.strip()] = value


def get_api_key():
    """Get YouTube API key from environment."""
    load_env()
    api_key = os.environ.get("YOUTUBE_API_KEY")
    if not api_key:
        print("ERROR: YOUTUBE_API_KEY environment variable not set", file=sys.stderr)
        sys.exit(1)
    return api_key


def get_channel_id(youtube, handle):
    """Get channel ID from handle."""
    # Search for channel by handle
    request = youtube.search().list(
        part="snippet",
        q=handle,
        type="channel",
        maxResults=1
    )
    response = request.execute()

    if not response.get("items"):
        print(f"ERROR: Channel not found: {handle}", file=sys.stderr)
        sys.exit(1)

    return response["items"][0]["snippet"]["channelId"]


def load_existing_videos():
    """Load existing videos from JSON file."""
    videos_path = DATA_DIR / "videos.json"
    if videos_path.exists():
        with open(videos_path, encoding="utf-8") as f:
            return json.load(f)
    return []


def get_latest_publish_date(videos):
    """Get the most recent publish date from existing videos."""
    if not videos:
        return None
    dates = [v.get("publish_date", "") for v in videos if v.get("publish_date")]
    return max(dates) if dates else None


def get_channel_videos(youtube, channel_id, published_after=None):
    """Get videos from a channel, optionally only after a certain date."""
    videos = []
    next_page_token = None

    # Build search params
    search_params = {
        "part": "snippet",
        "channelId": channel_id,
        "type": "video",
        "order": "date",
        "maxResults": 50,
    }

    if published_after:
        # YouTube API requires RFC 3339 format
        search_params["publishedAfter"] = f"{published_after}T00:00:00Z"

    while True:
        if next_page_token:
            search_params["pageToken"] = next_page_token

        request = youtube.search().list(**search_params)
        response = request.execute()

        video_ids = [item["id"]["videoId"] for item in response.get("items", [])]

        if video_ids:
            # Get detailed video info including duration
            details_request = youtube.videos().list(
                part="snippet,contentDetails,statistics",
                id=",".join(video_ids)
            )
            details_response = details_request.execute()

            for item in details_response.get("items", []):
                video = {
                    "id": item["id"],
                    "title": item["snippet"]["title"],
                    "description": item["snippet"]["description"],
                    "publish_date": item["snippet"]["publishedAt"][:10],
                    "duration": item["contentDetails"]["duration"],
                    "url": f"https://www.youtube.com/watch?v={item['id']}",
                    "thumbnail_url": item["snippet"]["thumbnails"].get("high", {}).get("url", ""),
                    "thumbnail": f"assets/thumbnails/{item['id']}.jpg",
                    "view_count": int(item["statistics"].get("viewCount", 0)),
                    "topics": extract_topics(item["snippet"]["title"], item["snippet"]["description"])
                }
                videos.append(video)

        next_page_token = response.get("nextPageToken")
        if not next_page_token:
            break

    return videos


def extract_topics(title, description):
    """Extract topic tags from title and description."""
    topics = []
    keywords = [
        "odoo", "erp", "inventory", "inventario", "manufacturing", "fabricacion",
        "mrp", "purchase", "compras", "sales", "ventas", "accounting", "contabilidad",
        "pos", "punto de venta", "website", "ecommerce", "crm", "project", "proyectos",
        "hr", "recursos humanos", "warehouse", "almacen", "quality", "calidad",
        "maintenance", "mantenimiento", "plm", "bom", "rutas", "workflow"
    ]

    text = (title + " " + description).lower()
    for keyword in keywords:
        if keyword in text:
            topics.append(keyword)

    return list(set(topics))[:10]  # Limit to 10 topics


def download_thumbnail(video_id, url):
    """Download thumbnail image."""
    if not url:
        return False

    filepath = THUMBNAILS_DIR / f"{video_id}.jpg"
    try:
        urllib.request.urlretrieve(url, filepath)
        return True
    except Exception as e:
        print(f"  Warning: Failed to download thumbnail for {video_id}: {e}", file=sys.stderr)
        return False


def main():
    try:
        from googleapiclient.discovery import build
    except ImportError:
        print("ERROR: google-api-python-client not installed", file=sys.stderr)
        print("Run: pip install google-api-python-client", file=sys.stderr)
        sys.exit(1)

    api_key = get_api_key()

    # Create directories
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    THUMBNAILS_DIR.mkdir(parents=True, exist_ok=True)

    # Load existing videos for incremental update
    existing_videos = load_existing_videos()
    existing_ids = {v["id"] for v in existing_videos}
    latest_date = get_latest_publish_date(existing_videos)

    print(f"Connecting to YouTube API...")
    youtube = build("youtube", "v3", developerKey=api_key)

    print(f"Finding channel: {CHANNEL_HANDLE}...")
    channel_id = get_channel_id(youtube, CHANNEL_HANDLE)
    print(f"  Channel ID: {channel_id}")

    if latest_date and existing_videos:
        print(f"Incremental update: fetching videos after {latest_date}...")
        print(f"  Existing videos: {len(existing_videos)}")
    else:
        print("Full fetch: no existing videos found...")

    new_videos = get_channel_videos(youtube, channel_id, published_after=latest_date)
    print(f"  Found {len(new_videos)} new videos from API")

    # Filter out duplicates (videos we already have)
    truly_new = [v for v in new_videos if v["id"] not in existing_ids]
    print(f"  Truly new videos: {len(truly_new)}")

    # Download thumbnails only for new videos
    if truly_new:
        print("Downloading thumbnails for new videos...")
        for video in truly_new:
            success = download_thumbnail(video["id"], video.get("thumbnail_url"))
            if success:
                print(f"  Downloaded: {video['id']}")
            video.pop("thumbnail_url", None)

    # Remove thumbnail_url from API response
    for video in new_videos:
        video.pop("thumbnail_url", None)

    # Merge: new videos + existing (avoiding duplicates)
    merged_videos = truly_new + existing_videos

    # Sort by publish date descending
    merged_videos.sort(key=lambda x: x["publish_date"], reverse=True)

    # Save to JSON
    output_path = DATA_DIR / "videos.json"
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(merged_videos, f, ensure_ascii=False, indent=2)

    print(f"\nTotal videos: {len(merged_videos)} ({len(truly_new)} new)")
    print(f"Saved to {output_path}")


if __name__ == "__main__":
    main()
