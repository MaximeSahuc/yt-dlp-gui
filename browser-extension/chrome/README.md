# MP3 Buddy - Browser Extension

A small companion extension for [MP3 Buddy](../../README.md). It adds a button to your browser that
sends the page you're viewing - together with the cookies needed to download it - straight to the
MP3 Buddy desktop app. No copy-pasting links, no manually exporting cookies.

Works in Chrome, Edge, Brave, Vivaldi, and other Chromium-based browsers.

## Quick install

1. Use Chrome, Edge, Brave, Vivaldi, or another Chromium-based browser.
2. Download & install MP3 Buddy from the [**Releases page**](https://github.com/MaximeSahuc/mp3-buddy/releases/latest).
3. Download the extension: [**mp3-buddy-extension-chrome.zip**](https://github.com/MaximeSahuc/mp3-buddy/releases/latest/download/mp3-buddy-extension-chrome.zip) and unzip it.
4. Type `chrome://extensions` in the address bar (the URL bar at the top, or `edge://extensions`, …), press Enter, then turn on **Developer mode** (top-right).
5. Click **Load unpacked** and select the unzipped folder.

Then open a video page and click the icon → **Send to app**. *(On Linux the app auto-launches; on Windows/macOS keep it running.)*

---

## More details

### What it does

- **One-click send** - click the toolbar icon to send the current video page to the app.
- **Right-click menu** - send a page, a video link, or selected text containing a URL.
- **Sends cookies too** - needed for login-required, age-restricted, or members-only videos.
  Cookies are read locally and handed to the app directly; they are never uploaded anywhere.
- **Smart badge** - the icon lights up automatically when you're on a supported video site.
- **Auto theme** - the popup follows your system's light / dark mode.

### How it works

The extension talks to the desktop app through a local link scheme (`mp3buddy://`). When you send
a page, your browser hands that link to MP3 Buddy, which comes to the front with the URL and cookies
already filled in.

> [!IMPORTANT]
> The MP3 Buddy desktop app must be **installed** for the extension to work - it's the app that
> receives the link. It doesn't need to be running first: on Linux the link launches it
> automatically. (On Windows/macOS, keep it running if the link doesn't start it.)

### Usage

1. Open a supported video page (YouTube, Bilibili, Twitch, Vimeo, Twitter/X, TikTok, Instagram,
   Facebook, Reddit, SoundCloud, and more).
2. Either:
   - Click the MP3 Buddy icon in the toolbar, then **Send to app**, or
   - Right-click the page or a video link and choose **Send to MP3 Buddy**.
3. The desktop app pops to the front with the link and cookies ready to download.

### Supported sites

The badge activates on the sites listed under `host_permissions` in
[`manifest.json`](./manifest.json) - including YouTube, Bilibili, Twitch, Vimeo, Dailymotion,
Niconico, Twitter/X, Instagram, TikTok, Facebook, Reddit, SoundCloud, Bandcamp, and Crunchyroll.
You can still send links from other sites via the right-click menu; downloading then depends on
yt-dlp's own [supported-site list](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md).

### Privacy

The extension only reads cookies when you explicitly send a page, and only for that page's site.
Those cookies are passed to the local desktop app through the `mp3buddy://` scheme and are **never
sent to any server**.
</content>
