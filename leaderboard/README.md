# SkyPop global scoreboard

Cloudflare Worker + D1, adapted from the public Nudge leaderboard structure. The game submits only typed name, score, mode and version. This project has one mode (`adventure`), so a single top 100 ranks each uppercase name’s best score. Names are not authenticated identities. Two people using the same name share a ranking.

## First deployment (user performs login and deployment)

Do not resolve, curl, browse or otherwise probe the new subdomain before this succeeds: the owner's router caches negative DNS answers.

```bash
cd /path/to/skypop/leaderboard
npm ci && npx wrangler login
./deploy.sh
```

The first line after `cd` installs the pinned tools and lets you log in. The second publishes the service. Select the Cloudflare account that owns quietbytehq.uk. The script creates/reuses `skypop-scores`, applies schema.sql, replaces the placeholder database ID, and deploys the Worker, custom domain and secrets together. It does not probe the public hostname. Reruns preserve the secret values and scores.

The admin URL is saved privately to `~/.config/skypop-scores-admin.txt`. The 256-bit random admin key and IP salt are stored in Cloudflare Worker secrets; a mode-0600 recovery copy is at `~/.config/skypop-scores/secrets.json`. Private deployment errors are in that directory's deploy.log. Do not paste or commit these files. If the admin link exists but the secrets copy is missing, deployment stops rather than rotating the key. Keep a private backup. The database ID written to wrangler.toml is not a secret and can be committed after deployment.

## Local checks

```bash
cd /path/to/skypop/leaderboard
npm ci
npm test
```

Tests launch `wrangler dev --local` on 127.0.0.1:8793 with a disposable local D1 database and random test secrets outside the repository. They never access the future domain or Cloudflare account. The local server and temporary files are cleaned up.

## After deployment

```bash
cd /path/to/skypop/leaderboard
python3 verify_live.py --deployed
```

This explicitly opts into the first public-domain request. It posts one temporary QA score, confirms storage and public appearance when it ranks, deletes only that submission through admin, then checks removal. On a fresh deployment the board must end empty. It never clears genuine players' entries. After this, run the new game, finish a qualifying run, enter a test name with online posting enabled, confirm it appears, and remove that specific test through admin. A name outside the top 100 can still be seen in the private recent-entry list. Live checks have not been performed before deployment.

## API and abuse controls

- `GET /api/top`: top 100, each name's best score. Equal scores sort by name for stable ordering.
- `POST /api/submit`: `{name, score, mode, version}`. Name is uppercase, stripped to A–Z, 0–9, space, dot, underscore and hyphen, limited to 12. Blocked substrings are checked with separators removed, both literally and with 0/1/3/4/5 mapped to o/i/e/a/s.
- Integer scores only; variables MIN_SCORE=1000, MAX_SCORE=1000000; score must be a multiple of 50. JSON numeric strings are refused. Only the `adventure` mode is accepted. Version is a bounded string.
- Six submission attempts per hour per salted SHA-256 address hash. Count-and-insert is atomic. Deleting a score does not erase rate-limit records. No raw addresses are stored by this application. Old hashes are removed on subsequent submissions after their hour expires; an idle database may retain expired hashes until the next submission.
- `/admin?key=…`: recent entries and delete controls. Missing/wrong key returns 404. Deletion uses a confirmation and authenticated POST so link previews cannot delete scores. Keep the URL private; whoever has it can moderate. No-referrer, no-store and a restrictive Content Security Policy are set.
- Prepared SQL with bound values only; UI data is inserted with textContent. No external fonts, trackers or embedded resources. Unexpected failures return only `server error` without details.

The score ceiling is intentionally generous, not a plausibility/cheat detector. There are at most 72 initial balls over ten stages, each at most 15 pops (100 each), at most 360 pickups (150 each), 10,500 total stage bonuses and up to 2,500 special points: a generous bound is 175,000. Failed-stage points are rolled back. No play-time or level-reached check is added. Scoring changes require revisiting the range and increment.

Fake scores remain possible: the game and endpoint are public, there is no client secret, and no scores are claimed verified. Salted IP limits can be evaded by changing address and can affect users behind a shared address. Same-name impersonation is possible. Cloudflare processes network addresses to serve requests; our database stores only a temporary salted hash. Unlike Nudge's original, this implementation uses a full SHA-256 hash, atomic limits, separate rate rows, POST deletion and DOM-built admin rows.

The client never waits for submission, retries offline posts, or uploads the local score history. F2 / gamepad X / the privacy control remembers local-only. Only explicitly saving a newly finished qualifying run sends a post. A failed post does not prevent local saving or restart.
