# Clockwise

A command-line tool that reads Emacs org-mode files and aggregates
clock entries, displaying total time spent within a specified date
range.

## Motivation

I built this to keep track of the time I spend working on my
freelance/consulting projects. I know org-clock-table exists but I
wanted a way to simply specify a time duration and have it output the
time spent in hours and minutes.

Much of the code is written with the help of claude and other AI
assistants. The initial version of this tool was a "one-shot" vibe
coded python script, which at one point started getting difficult to
iterate on without the AI agents going around in circles. So I ported
it to rust and wrote some of the fundamental abstractions by hand.

### Quick example

```bash
$ clockwise -t jan..=feb -b week --heatmap myproject.org
------------------------------
    DeJan     Feb
Mon   ▓ . ▓ ▒ █ ▓ █ █
Tue   ▒ █ ▓ █ ▓ ▓ █ █
Wed   ▒ ▓ ▓ ▒ █ █ █ █
Thu ▓ . ▓ ▓ █ █ █ ▓ ▓
Fri ▓ . . ▓ ▒ ▒ ▒ ▓ ▓
Sat . . . . . . . . .
Sun . . . . . . . .
------------------------------
From 2026-01-01 to 2026-03-01
------------------------------
W01 08:30
W02 09:43
W03 14:23
W04 26:17
W05 25:10
W06 30:33
W07 30:08
W08 30:50
W09 32:22
------------------------------
Total: 207 hours 56 minutes
```

## Installation

```bash
cargo install --git https://github.com/naiquevin/clockwise.git
```

## Usage

```bash
clockwise <file.org> [--time-duration <value>] [--breakdown <value>] [--heatmap]
```

Absolute and relative paths are supported for the file path, including
`~/`.

## Options

### `--time-duration` / `-t`

Specifies the date range to summarize. Defaults to `d` (today).

**Days**

| Value | Meaning    |
|-------|------------|
| `d`   | Today      |
| `d-1` | Yesterday  |
| `d-N` | N days ago |

**Weeks**

| Value | Meaning                                      |
|-------|----------------------------------------------|
| `w`   | Current week (Mon–Sun)                       |
| `w-N` | N weeks ago                                  |
| `wN`  | Nth ISO week of the current year (e.g. `w6`) |

**Months**

| Value       | Meaning                                 |
|-------------|-----------------------------------------|
| `m`         | Current month                           |
| `m-N`       | N months ago                            |
| `jan`–`dec` | Named month (year inferred — see below) |

**Quarters**

| Value     | Meaning                         |
|-----------|---------------------------------|
| `q1`–`q4` | Nth quarter of the current year |

**Day of week**

| Value           | Meaning                      |
|-----------------|------------------------------|
| `mon`–`sun`     | That day of the current week |
| `mon-1`–`sun-1` | That day of last week        |

**Absolute dates**

| Value        | Meaning         |
|--------------|-----------------|
| `2026-02-05` | A specific day  |
| `2026-02`    | An entire month |

**Ranges**

Use `..` (end excluded) or `..=` (end included) to specify a
range. Both sides accept any of the short forms above.

```
2025-07-01..2025-07-20     # July 1–19
2025-07-01..=2025-07-20    # July 1–20
mon..wed                   # Monday up to (not including) Wednesday
mon..=fri                  # Monday through Friday
jan..=mar                  # January through March
```

**Year inference for short forms**

When no year is given, the year is inferred: if the named period
(month, weekday, etc.) is still in the future relative to today, the
previous year is used; otherwise the current year is used. The
exception is December when today is in December — that always refers
to the current year.

### `--breakdown` / `-b`

Breaks down the output into sub-periods. Valid values: `d` (day), `w`
(week), `m` (month), `q` (quarter). The breakdown must be smaller than
the time duration, otherwise the option is ignored.

### `--heatmap`

Displays a basic github-like calendar heatmap.

## Examples

```bash
# Yesterday's total time
clockwise --t d-1 ~/myproject.org

# This week broken down by day
clockwise --t w --b day ~/myproject.org

# January through March (inclusive)
clockwise --t jan..=mar --heatmap ~/myproject.org

# Last Monday to last Friday
clockwise --t mon-1..=fri-1 ~/myproject.org
```

## Developer documentation

- Can be found in the `dev-docs` dir
- Mainly for documenting design decisions etc.
- For human use but coding assistants can and should refer to
  it. Prefer reading only the minimum number of docs required to
  perform the task
- One common workflow is to write a dev-doc and ask the coding
  assistant to treat it like a prompt. Once the task is performed,
  modify the dev doc to make it read more like a doc or a spec instead
  of a prompt.
