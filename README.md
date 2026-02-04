# rusty-lens

rusty-lens is a TUI for browsing **Microsoft Defender for Endpoint device timeline** CSV exports, written in [Rust](https://rust-lang.org/). Built for DFIR (digital forensics and incident response): stream and filter large timeline CSVs with vim-style keys and a terminal UI.

---

## install

```console
cargo install rusty-lens --git https://github.com/shalloran/rusty-lens
```

Then run:

```console
rusty-lens /path/to/timeline.csv
```

---

## use

rusty-lens is modal. The command bar at the bottom shows the current mode (e.g. **NORMAL**, **SEARCH**, **FILTER**, **TIME**) and context-sensitive key hints.

### controls – normal mode

- `j` / `Down` — next event
- `k` / `Up` — previous event
- `Page Up` / `Page Down` — scroll the detail panel
- `/` — start search (type query, then Enter to apply, Esc to cancel)
- `a` — filter by action type (pick from list, Enter to apply, Esc to clear)
- `t` — filter by time range (presets or custom picker)
- `x` — clear all filters and search
- `q` / `Esc` — quit

### search (`/`)

In **SEARCH** mode, type your query and press Enter. Search is case-insensitive and multi-word: space-separated terms are ANDed across event fields (e.g. file names, paths, command lines, hashes). Esc cancels without applying.

### action type filter (`a`)

In **FILTER** mode, the event list is replaced by a list of action types present in the data (e.g. ProcessCreated, ConnectionSuccess). Move with `j`/`k`, press Enter to apply that filter. Esc clears the action-type filter and returns to the event list.

### time filter (`t`)

In **TIME** mode you can narrow events by time range.

1. **Presets** — Pick one of: Today, Yesterday, Last 24 hours, Last 7 days, Last 30 days. `j`/`k` to move, Enter to apply.
2. **Custom (pick dates from data)** — Pick start date, then start hour, then end date, then end hour from dates/hours that actually appear in the timeline. For single-date timelines you go straight to start hour → end hour. All choices are constrained to the data so you can’t pick invalid ranges.
3. **Custom (type range)...** — Type a time expression and Enter to apply. Supported:
   - `clear` — remove time filter
   - Relative: `today`, `yesterday`, `last 7 days`, `last 24h`, etc.
   - Absolute: `after 2025-01-15`, `before 2025-02-01`, `2025-01-20 to 2025-01-25`

Esc steps back (e.g. from end hour to end date to start hour to start date to presets) or exits time filter.

### layout

- **Left** — Event list: time | action type | file or process. Shows up to 5000 filtered events; count in the title.
- **Right** — Detail panel: all non-empty fields for the selected event, with wrapping and a vertical scrollbar.
- **Bottom** — Command bar: current mode and key hints.

### theme

Leet haxor-style palette: black background, bright green text, cyan titles and highlight, green borders, black-on-green command bar.

---

## csv format

Expects a **Defender device timeline export** CSV: one header row with 66 columns (e.g. Event Time, Machine Id, Computer Name, Action Type, File Name, …), one event per row. RFC 4180 style: quoted fields and embedded commas are supported. Malformed rows are skipped. Loading is streamed with a default cap of 100,000 rows.

---

## tests

```console
cargo test
```

Uses `tests/fixture_sample.csv`, a minimal 66-column, two-row fixture with no real data.

---

## design

rusty-lens is a [ratatui](https://crates.io/crates/ratatui) app using [crossterm](https://crates.io/crates/crossterm). The parser uses the [csv](https://crates.io/crates/csv) and [serde](https://crates.io/crates/serde) crates; timestamps and time ranges use [chrono](https://crates.io/crates/chrono). Timeline CSV is read from disk and held in memory up to the row cap (no external services or credentials).

---

## license

GPLv3, see [LICENSE](LICENSE).
