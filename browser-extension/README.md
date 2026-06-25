# MP3 Buddy — Browser Extension

A small companion extension for [MP3 Buddy](../README.md). It adds a button to your browser that
sends the page you're viewing — together with the cookies needed to download it — straight to the
MP3 Buddy desktop app. No copy-pasting links, no manually exporting cookies.

Works in Chrome, Edge, Brave, Vivaldi, and other Chromium-based browsers.

## What it does

- **One-click send** — click the toolbar icon to send the current video page to the app.
- **Right-click menu** — send a page, a video link, or selected text containing a URL.
- **Sends cookies too** — needed for login-required, age-restricted, or members-only videos.
  Cookies are read locally and handed to the app directly; they are never uploaded anywhere.
- **Smart badge** — the icon lights up automatically when you're on a supported video site.
- **Auto theme** — the popup follows your system's light / dark mode.

## How it works

The extension talks to the desktop app through a local link scheme (`mp3buddy://`). When you send
a page, your browser hands that link to MP3 Buddy, which comes to the front with the URL and cookies
already filled in.

> [!IMPORTANT]
> The MP3 Buddy desktop app must be **installed and running** for the extension to work — it's the
> app that receives the link.

## Install

The extension is bundled with the app, so there's nothing extra to download.

1. In MP3 Buddy, open **Toolbox → Browser Extension** and click **Open extension folder**
   (this folder, `browser-extension/`).
2. In your browser, go to `chrome://extensions` (or `edge://extensions`, `brave://extensions`, …).
3. Turn on **Developer mode** (top-right toggle).
4. Click **Load unpacked** and select the folder from step 1.
5. Pin the **MP3 Buddy Helper** icon next to the address bar.

## Use

1. Open a supported video page (YouTube, Bilibili, Twitch, Vimeo, Twitter/X, TikTok, Instagram,
   Facebook, Reddit, SoundCloud, and more).
2. Either:
   - Click the MP3 Buddy icon in the toolbar, then **Send to app**, or
   - Right-click the page or a video link and choose **Send to MP3 Buddy**.
3. The desktop app pops to the front with the link and cookies ready to download.

## Supported sites

The badge activates on the sites listed under `host_permissions` in
[`manifest.json`](./manifest.json) — including YouTube, Bilibili, Twitch, Vimeo, Dailymotion,
Niconico, Twitter/X, Instagram, TikTok, Facebook, Reddit, SoundCloud, Bandcamp, and Crunchyroll.
You can still send links from other sites via the right-click menu; downloading then depends on
yt-dlp's own [supported-site list](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md).

## Privacy

The extension only reads cookies when you explicitly send a page, and only for that page's site.
Those cookies are passed to the local desktop app through the `mp3buddy://` scheme and are **never
sent to any server**.
</content>
