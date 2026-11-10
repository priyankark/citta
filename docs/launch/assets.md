# Launch media

Prepare a square **240 × 240 PNG logo** under 2 MB and **four 1270 × 760 gallery PNGs**. Keep the text legible on a phone. Show the actual product and a clear task handoff. These dimensions follow [Product Hunt's media guidance](https://help.producthunt.com/en/articles/17834537-4-build-your-product-page).

## Gallery sequence

| Image | Headline | Content |
| --- | --- | --- |
| 1 | Your agent. All your devices. | A simple laptop ↔ desktop diagram with four reasons: files, tools, hardware, compute. Supporting text: “Send a task to the computer that has what it needs.” |
| 2 | The receipts are on the other Mac. | Real expense report crop with a clear Mini → Air → report flow. Call out duplicate found and missing receipt flagged. Label “Real execution · synthetic text receipts.” |
| 3 | Now use the Mini's local model. | Real model-result crop with Air → Mini → categories flow. Label “Gemma 3 4B via Ollama.” Mention that Codex orchestration used a cloud service, so the image does not imply the full workflow was local. |
| 4 | Start with two computers. | “Connect Tailscale. Set up Citta. Try one task.” Explain that each device needs the tools and permissions for its part. Include “Free for permitted noncommercial use.” |

Avoid exposing device addresses, usernames, personal paths, tokens or browser tabs. Do not draw invented successful UI states. Any diagram is explanatory artwork and should remain distinguishable from the actual UI.

## Existing video

The completed local film is about 1 minute 51 seconds, 1920 × 1080 H.264/AAC, with captions and narration. It contains real task execution and illustrative later scenarios. It has not been publicly hosted for this launch.

Release-media source: `local/product-video/edit-v3/final/citta-expenses-and-local-ai.mp4` (ignored production material). Use the matching captions and production notes. Do not commit private execution records or the Clipchamp editing session to the public repository.

**Editorial choice:** the current opening says “work trip.” The technical example uses synthetic receipts and no reimbursement was submitted. The film can remain as recorded if its adjacent launch-page caption makes the license explicit. Use: **“Demonstration with synthetic data. Citta is available under PolyForm Noncommercial 1.0.0; commercial use is not licensed. See the license for permitted uses.”** Reframing the narration and matching captions as a personal trip is an alternative, not a release requirement. Do not describe the existing export as revised if it has not changed.

**Truth boundaries:** waiting is shortened; the Air could not save its report because its disk was full, and the returned report was saved on the Mini. Exact totals came from Python, categories and per-expense notes from Gemma. Local inference took 12 seconds in one recorded task; that is not an end-to-end or general performance claim. Xcode/phone, transcription and camera examples are illustrative. No native Air desktop recording or OCR was demonstrated.

## Before uploading

- [ ] Approved logo and rendered gallery files exist and have been visually reviewed at their final sizes.
- [ ] Actual UI crops contain only intended public demo data.
- [ ] The video has the adjacent noncommercial license caption above, or its narration and captions have been revised to personal travel.
- [ ] Upload the final video only after the public media destination is chosen; put its viewer link in `listing.json`.
- [ ] Inspect the video from a signed-out browser with audio and captions enabled.
- [ ] Add concise alt text/captions where the launch editor supports them.

This folder contains the media brief; it does not imply assets were uploaded or a launch scheduled.
