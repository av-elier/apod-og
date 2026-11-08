# apod-og

## What this codebase does
- Proxies NASA's Astronomy Picture of the Day (APOD) and The Cat API to inject Open Graph (OG) meta tags for embedding in Slack, social media, or other platforms.
- Serves a minimal HTML page with OG tags pointing to the daily image or video thumbnail.
- Provides a temporary redirect (HTTP 307) to the APOD image for backward compatibility.
- Caches responses for 1 hour to reduce upstream API calls.
- Handles UTF-16LE responses from NASA APOD (malformed Content-Type).
- Supports both images and YouTube videos (extracts thumbnails).

## Auth shape
- **No authentication**: All endpoints are public.
- **CAT_API_KEY**: Optional environment variable for The Cat API (defaults to a demo key).
- **PORT**: Optional environment variable for binding the HTTP server (defaults to 8080).
- **User-Agent**: Hardcoded in requests to upstream APIs.

## Threat model
- **Upstream dependency risks**: Relies on NASA APOD and The Cat API for content. Malformed or malicious responses could break parsing or inject unintended content.
- **Cache poisoning**: 1-hour caching of upstream responses could amplify malicious payloads.
- **Open redirect**: The `/307` endpoint redirects to the APOD image URL, which could be abused if the upstream URL is attacker-controlled.
- **SSRF**: The application makes HTTP requests to user-controlled input (e.g., YouTube video URLs) but restricts domains to `apod.nasa.gov` and `youtube.com`.
- **Environment variables**: CAT_API_KEY and PORT are read from the environment, risking exposure if misconfigured.

## Project-specific patterns to flag
- **UTF-16LE handling**: Custom logic for malformed NASA APOD responses (fallback to UTF-16LE if UTF-8 fails).
- **YouTube ID extraction**: Regex-based parsing of YouTube URLs to extract video IDs for thumbnails.
- **HTML scraping**: Uses `scraper` crate to parse APOD HTML for image/video links.
- **Caching**: Uses `cached` crate with a 1-hour TTL for upstream responses.
- **Open Graph injection**: Hardcoded HTML template with dynamic OG tags.

## Known false-positives
- **YouTube regex patterns**: Hardcoded regex for YouTube URLs may flag as "regex injection" but is safe (no user input is interpolated).
- **UTF-16LE fallback**: May flag as "encoding confusion" but is intentional for NASA APOD.
- **Environment variables**: CAT_API_KEY and PORT may flag as "secrets in code" but are optional/configurable.
- **Open redirect**: `/307` may flag as "unsafe redirect" but only redirects to NASA APOD images.
