# SkyPop 1.1.0 — The Shattered Sky (prepared locally)

## Features

Three illustrated opening cards introduce the shattered Sky Crystal and the explorer's ten-world journey. Three victory cards reunite the crystal, celebrate the restored worlds and award the Guardian of the Skies title. Existing world artwork, character art, victory animation and licensed audio are reused. Enter/Space/A/click advances; Escape/B skips. The scene input grace period prevents an initial held action from skipping a card or firing a rope. Defeat uses an encouraging message, without showing the victory ending.

The optional Adventure global top 100 accepts scores from 1,000 to 1,000,000 in multiples of 50. Each name's highest score ranks once. Name entry discloses the destination and the four transmitted fields; F2/gamepad X/click toggles a remembered local-only choice. Mouse score submission now requires clicking the save control, so changing the privacy setting cannot accidentally submit. HTTP requests are asynchronous, limited to five seconds and discarded on failure without retries or local-history uploads. Snap declares network access.

## Checks performed

- Existing Godot game regression suite: 43,443 assertions passed without licensed audio; 43,484 passed with the existing licensed MP3 files.
- New story/online tests: 20 checks passed, covering opening/ending progression, skip/grace behavior, frozen gameplay, preserved score, defeat route, remembered local-only preference, no request in local-only mode, four-field payload, actual localhost HTTP request, immediate continuation and offline cleanup.
- Worker tested using real `wrangler dev --local` and disposable D1: valid score and best-per-name, below minimum, impossible maximum, wrong increment, fractional/string score, blocked leetspeak name, unknown/prototype/array mode, malformed body, ten concurrent requests limited to six successes, admin with/without key, delete and empty-board confirmation, hash storage and safe page construction. An actual Godot game submitted GAMEQA to this local Worker and it appeared in `/api/top` before admin removal. Unexpected server errors return a generic response.
- Opening, victory ending and online/local-only name-entry screens rendered and visually inspected at 1280×720. The actual scoreboard HTML was rendered locally in a headless browser using staged sample rows; its public-domain version was not opened.
- Linux x86-64 and Windows x86-64 exports succeeded. The Linux release executable passed a headless startup smoke check. Windows execution has not been tested on Windows. No new Snap was published or installed; its network permission is prepared in the recipe.
- Some audio-loaded Godot tests and forced startup exits report ObjectDB/resource references at shutdown. Assertions pass; these are reported rather than counted as clean teardown.

## Not performed yet

No GitHub push, Cloudflare login, D1 remote creation, Worker deployment, public-subdomain lookup, live score submission, live moderation test or live game submission has been performed. The user runs login/push/deploy. After first deployment, run `leaderboard/verify_live.py --deployed`, then finish a qualifying run in the new game and remove its test entry through the private admin page.

The production key and IP salt are generated only by the deploy script, stored in Worker secrets, and saved with private permissions outside the repository. Local tests use disposable random test values only. Build outputs, licensed MP3 files, dependencies, caches and private configuration are ignored.

## Limits

Fake scores remain possible. The public endpoint accepts self-reported scores and has no client secret or proof of play. Range/increment validation, name filtering, salted address limits and moderation deter junk; they do not authenticate players or stop an attacker changing addresses. People sharing an address share the limit; people sharing a name share a ranking. No play-time or level-reached plausibility check was added.

Deploy the scoreboard and complete the pending live checks before publishing or installing this update for general play. See `leaderboard/README.md` for the two-command deployment flow and private admin-file location.
