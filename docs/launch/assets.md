# Launch media

Updated November 12: a **240 × 240 PNG logo**, **four 1270 × 760 gallery PNGs** and a **1280 × 720 demo thumbnail**. The gallery and thumbnail now show a network of agents sharing access to tools, data and compute. Dimensions and checksums are in the manifest. An older gallery is saved on Product Hunt; replacement upload of this revision is pending. The launch has not been scheduled or published.

Use [the logo](gallery/logo.png) and the four images below. [The asset manifest](gallery/manifest.json) includes dimensions, checksums, alt text, captions and result-crop provenance. Editable sources are [HTML](gallery/gallery.html), [CSS](gallery/gallery.css), and [SVG](gallery/logo.svg); the logo reuses Citta's existing panel mark. Render them with `node docs/launch/gallery/render.cjs` from an environment with Playwright and Chromium. This uses a separate headless browser, with no signed-in profile.

## Gallery sequence

Lead with the network of agents, then show what its members can do with data, tools and compute. The gallery and thumbnail have been regenerated and visually reviewed; external replacements require separate verification.

| Image | Headline | Content |
| --- | --- | --- |
| [1](gallery/01-your-devices.png) | A network for your AI agents. | Three connected agents with private data, installed tools and spare compute. Hardware access is another reason to use the network. |
| [2](gallery/02-the-files.png) | The network has the files you need. | Actual receipt-report results from the Air returned to the Mini. Duplicate excluded; missing receipt flagged. Synthetic data labeled. |
| [3](gallery/03-the-model.png) | Put a local model to work. | Actual Gemma result table from the Mini. Local inference and cloud-backed Codex orchestration are distinguished. |
| [4](gallery/04-get-started.png) | Build your agent network. | Connect Tailscale, install and set up Citta, then try a small task. Each member needs its tools, accounts and permissions. |

The [demo thumbnail](../media/demo-thumbnail.png) uses the same agent-network diagram. Its editable source is [demo-thumbnail.html](gallery/demo-thumbnail.html). YouTube title and description are in [video.md](video.md).


Avoid exposing device addresses, usernames, personal paths, tokens or browser tabs. Do not draw invented successful UI states. Any diagram is explanatory artwork and should remain distinguishable from the actual UI.

## Existing video

The completed local film is about 1 minute 51 seconds, 1920 × 1080 H.264/AAC, with captions and narration. It contains real task execution and illustrative later scenarios. It is hosted as an [unlisted YouTube video](https://www.youtube.com/watch?v=EPdPu5djxSQ). Signed-out playback was verified, and all 45 manual-caption cues were independently matched to the source SRT. Product Hunt attachment persistence still needs a saved-preview check. The new YouTube title, description and thumbnail require a verified external save.

Release-media source: `local/product-video/edit-v3/final/citta-expenses-and-local-ai.mp4` (ignored production material). Use the matching captions and production notes. Do not commit private execution records or the Clipchamp editing session to the public repository.

**Editorial choice:** the current opening says “work trip.” The technical example uses synthetic receipts and no reimbursement was submitted. The film can remain as recorded if its adjacent launch-page caption makes the license explicit. Use: **“Demonstration with synthetic data. Citta is available under PolyForm Noncommercial 1.0.0; commercial use is not licensed. See the license for permitted uses.”** Reframing the narration and matching captions as a personal trip is an alternative, not a release requirement. Do not describe the existing export as revised if it has not changed.

**Truth boundaries:** waiting is shortened; the Air could not save its report because its disk was full, and the returned report was saved on the Mini. Exact totals came from Python, categories and per-expense notes from Gemma. Local inference took 12 seconds in one recorded task; that is not an end-to-end or general performance claim. Xcode/phone, transcription and camera examples are illustrative. No native Air desktop recording or OCR was demonstrated.

## Media checks

- [x] Logo and rendered gallery files exist and have been visually reviewed at their final sizes.
- [x] Actual result crops contain only intended public synthetic demo data; no device addresses, usernames, personal paths, tokens, task IDs or browser tabs.
- [ ] Verify that Product Hunt saved the staged video and adjacent synthetic-data/noncommercial license description after global **Save changes**.
- [x] Upload the final video to the selected unlisted destination and put its viewer link in `listing.json`.
- [x] Verify signed-out video playback and independently match all 45 manual-caption cues to the source SRT.
- [ ] Add concise alt text/captions where the launch editor supports them.

This folder contains the media and its brief. The previously uploaded logo and gallery are saved in the external editor. The video is available unlisted, its Product Hunt attachment needs a saved-preview check, and the launch is not scheduled. Revised gallery uploads require their own verification. The original video and its detailed truth boundaries above are unchanged.
