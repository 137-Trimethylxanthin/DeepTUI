# DeepTUI

Terminal dashboard for your DeepSeek API usage — balance, tokens, and costs with live updates.

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.94%2B-orange" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License">
</p>

## Features

- **Live balance** — credits, granted vs topped-up, updated every 30s
- **Token usage** — today, this month, all-time, updated every 15min
- **Cost tracking** — monthly spend from the platform billing summary
- **Dual API** — balance from the official API, usage from the platform's internal endpoints
- **Single binary** — no runtime dependencies, no browser automation needed

## Screenshot

```
┌──────────────────────────────────────────────────┐
│ DeepSeek Credits                                  │
│ ┌ Balance ─────────────────────────────────────┐  │
│ │ Total:     $1.72  USD                        │  │
│ │ Granted:   $0.00                             │  │
│ │ Topped Up: $1.72                             │  │
│ │                                               │  │
│ │ Available: Yes                                │  │
│ └──────────────────────────────────────────────┘  │
│ ┌ Usage ───────────────────────────────────────┐  │
│ │ Today:     74.4M tokens                      │  │
│ │ Month:     139.2M tokens                     │  │
│ │ All Time:  139.2M tokens                     │  │
│ │ Balance:   $1.72                              │  │
│ └──────────────────────────────────────────────┘  │
│ [R]efresh  [Q]uit | bal:14:30 use:14:30 | 14:30 │
└──────────────────────────────────────────────────┘
```

## Quick start

```bash
git clone git@github.com:137-Trimethylxanthin/DeepTUI.git
cd DeepTUI
cp .env.example .env
# edit .env with your keys
cargo run
```

## Configuration

Copy `.env.example` to `.env`:

```env
# Required — get yours at https://platform.deepseek.com/api_keys
DEEPSEEK_API_KEY=sk-...

# Optional — for usage stats. Get from DevTools → Application →
# Local Storage → platform.deepseek.com → token / authToken / accessToken
DEEPSEEK_PLATFORM_TOKEN=...

# Poll intervals in seconds (defaults shown)
DEEPSEEK_BALANCE_SECS=30
DEEPSEEK_USAGE_SECS=900
```

Or export them directly:

```bash
export DEEPSEEK_API_KEY="sk-..."
export DEEPSEEK_PLATFORM_TOKEN="eyJ..."
cargo run
```

## Getting a platform token

The platform token unlocks usage stats (tokens per day, monthly totals, cost breakdown). To get one:

1. Log into [platform.deepseek.com](https://platform.deepseek.com)
2. Open DevTools (F12) → Application → Local Storage → `platform.deepseek.com`
3. Copy the value of `token`, `authToken`, or `accessToken`
4. Paste it as `DEEPSEEK_PLATFORM_TOKEN` in your `.env`

## Keys

| Key | `r` | `q` / `Esc` |
|---|---|---|
| Action | Force refresh all data | Quit |

## How it works

The app hits two separate DeepSeek APIs:

| Data | Endpoint | Auth | Refresh |
|---|---|---|---|
| Balance | `api.deepseek.com/user/balance` | API key (Bearer) | 30s |
| Summary | `platform.deepseek.com/api/v0/users/get_user_summary` | Platform token (Bearer) | 15min |
| Usage | `platform.deepseek.com/api/v0/usage/amount` | Platform token (Bearer) | 15min |

The platform endpoints are reverse-engineered from the web UI's JavaScript bundle — they're undocumented internal APIs used by the dashboard at [platform.deepseek.com/usage](https://platform.deepseek.com/usage).

## Build

```bash
cargo build --release
./target/release/deepCredits
```
