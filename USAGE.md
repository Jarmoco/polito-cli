# Usage Guide

## Quick Reference

```bash
polito login                              # authenticate
polito courses                            # list enrolled courses
polito courses --year 2024/2025           # filter by academic year
polito files 258674 --list                # browse file tree
polito files 258674 --path slides         # filter tree by path
polito files 258674 --path 33248890 -o ./dl  # download by file ID
polito files 258674 --all -o ./mats       # download all files
polito notices 258674                     # course notices
polito exams                              # upcoming exams
polito lectures                           # lecture timetable
polito lectures --from 2024-09-01 --to 2024-09-30
polito grades                             # recorded grades
polito clone 258674 -o ./sdp              # clone course files
polito mock                               # TUI with mock API
```

Add `--json` to any command for raw JSON output.

---

## Commands

### `polito login`

Prompts for username (s-number) and password, authenticates against the API, and stores the Bearer token in `~/.config/polito-cli/.token`.

```bash
polito login
```

### `polito logout`

Calls the server logout endpoint and clears the stored token and username files.

```bash
polito logout
```

### `polito whoami`

Displays the authenticated user's profile: name, username, email, and student ID.

```bash
polito whoami
```

### `polito courses`

Lists all enrolled courses in a formatted table. Use `--year` to filter by academic year.

```bash
polito courses
polito courses --year 2024/2025
polito courses --json
```

Columns: ID, Code (shortcode), Name, Teacher, Year.

### `polito files`

Browse or download course files. The `COURSE_ID` argument is required (find it via `polito courses`).

| Flag | Description |
|------|-------------|
| `--list` | Print file tree instead of downloading |
| `--path <FILE>` | Filter tree by name fragment, or download by file ID |
| `--output <DIR>` | Output directory for downloads |
| `--all` | Download all files recursively |
| `--size` | Show total file count and size |

```bash
# Print full file tree
polito files 258674 --list

# Print subtree matching name
polito files 258674 --list --path "slides"

# Download a single file by ID
polito files 258674 --path 33248890 --output ./downloads

# Download all files recursively
polito files 258674 --all --output ./course-materials

# Show total file count and size
polito files 258674 --size
polito files 258674 --size --json

# JSON output (prints tree as JSON)
polito files 258674 --json
```

If `--path` is provided without `--list`, the tool attempts to download by file ID. If the download fails (404), it falls back to printing the file tree as a hint.

### `polito notices`

Lists course notices with HTML stripped from the content.

```bash
polito notices 258674
polito notices 258674 --json
```

### `polito exams`

Lists upcoming exams with booking status and grades (if available).

```bash
polito exams
polito exams --json
```

Status colors:
- `BOOKED` — green
- `AVAILABLE` — yellow
- `UNAVAILABLE` — red
- Others — dim

### `polito lectures`

Shows the lecture timetable with optional date range and course filters.

| Flag | Description |
|------|-------------|
| `--from <DATE>` | Start date (YYYY-MM-DD) |
| `--to <DATE>` | End date (YYYY-MM-DD) |
| `--course-id <ID>` | Filter by course |

```bash
polito lectures
polito lectures --from 2024-09-01 --to 2024-09-30
polito lectures --course-id 258674
polito lectures --from 2024-09-01 --to 2024-09-30 --course-id 258674 --json
```

### `polito grades`

Shows recorded exam grades. Handles both numeric grades (28) and string grades ("30L").

```bash
polito grades
polito grades --json
```

Color coding: green (>= 28 or "30L"), yellow (>= 24), dim (below 24).

### `polito clone`

Git-like course clone for offline access. Downloads all course files into a local directory with checksum metadata tracking.

| Flag | Description |
|------|-------------|
| `-o, --output <DIR>` | Output directory (default: `polito-<course-name>`) |
| `--overwrite` | Overwrite changed files |
| `--skip-existing` | Keep local files, download only new ones |
| `--backup` | Backup changed files before overwriting |

```bash
# First run: clone all files
polito clone 258674 --output ./sdp

# Re-run detects conflicts and exits with a report
polito clone 258674 --output ./sdp
# → Conflict report printed, exits with code 1

# Resolve by overwriting all changed files
polito clone 258674 --output ./sdp --overwrite

# Keep local files, download only new ones
polito clone 258674 --output ./sdp --skip-existing

# Backup changed files before overwriting
polito clone 258674 --output ./sdp --backup
```

Clone creates a `.polito-clone.json` metadata file inside the output directory tracking checksums of all downloaded files. On re-run, it compares local checksums against the server and reports:
- **added** — new files on the server
- **changed** — files with different checksums
- **removed** — files deleted from the server

In CLI mode, conflicts only produce a report — the user must re-run with `--overwrite`, `--skip-existing`, or `--backup`.

### `polito mock`

Launches the TUI with a mock API URL prompt. Useful for testing against a local Prism mock server.

```bash
polito mock
# Prompts: Mock API URL:
# Enter: http://localhost:6509
```

### `polito --help`

Prints usage summary with all commands and flags.

### `polito --version`

Prints the binary version.

---

## TUI Mode

Run `polito` with no arguments to enter interactive mode. The TUI provides keyboard-driven navigation through all features.

### Key Bindings

| Key | Action |
|---|---|
| Up / Down arrows | Navigate lists |
| Enter | Select item / expand |
| `n` | View notices (from course list) |
| Esc | Go back / cancel |
| `q` | Quit TUI |

### Screens

| Screen | Description |
|--------|-------------|
| **Main Menu** | Courses, exams, grades, lectures, clone, quit |
| **Courses** | Enrolled course list; Enter to browse files, `n` for notices |
| **Files** | Hierarchical file tree with sizes; scroll to browse; Enter opens a modal with download/open options |
| **Notices** | Notice list; Enter to expand/collapse content |
| **Exams** | Exam table with color-coded booking status |
| **Lectures** | Week-by-week lecture timetable with day grouping |
| **Grades** | Grade table with color-coded values |
| **Clone** | Course selection → file selection (checkboxes, directory toggle) → clone execution with progress |

---

## Configuration

### Token Storage

The tool stores authentication tokens using the XDG Base Directory specification:

- **Primary**: `POLITO_TOKEN` environment variable
- **Fallback**: `~/.config/polito-cli/.token` file
- **Username**: `~/.config/polito-cli/.user` file

Setting `POLITO_TOKEN` directly is the recommended approach for scripting and CI:

```bash
export POLITO_TOKEN=your_token_here
polito courses
```

### API Endpoint

The default API URL is `https://app.didattica.polito.it/api`. To use a mock server, set the `POLITO_MOCK_URL` environment variable or use `polito mock`.

---

## Testing with Mock Server

### CLI:
```bash
# Terminal 1: start the Prism mock server
./prism.sh

# Terminal 2: run commands against the mock
POLITO_TOKEN=test cargo run -- courses --json
POLITO_TOKEN=test cargo run -- courses
POLITO_TOKEN=test cargo run -- files 258674 --list
POLITO_TOKEN=test cargo run -- clone 258674 --output ./test-clone
```

### TUI:

You can start a TUI session against a mock server by running:
```bash

# Terminal 1: start the Prism mock server
./prism.sh


# Terminal 2: run the TUI against the mock
polito mock     # you will be asked to enter the api url
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (network, auth, parse, config, or clone conflicts) |

---

## Build

```bash
cargo build --release
```

The binary is placed at `target/release/polito`.
