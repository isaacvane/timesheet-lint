# timesheet

A small command-line tool for a plain text time sheet format: it validates
a file and either pretty-prints it or emits JSON.

I keep my hours in a text file per pay period, one line per block of work.
It's fast to edit, but it's also easy to typo a time, leave an entry under
the wrong day, or double-book a block by accident. This tool catches that
before the numbers go into an invoice, and the `--json` mode exists so the
validated data can feed a small invoicing script without re-parsing text.

## The format

```
# 2026-09-15
09:00-12:30 acme/website     build the login page
13:15-17:00 acme/website     code review with the client
09:00-10:00 internal         standup

# 2026-09-16
09:00-11:00 acme/website     fix the layout bug on mobile
```

- A line starting with `#` opens a new day: `# YYYY-MM-DD`.
- Every other non-blank line is an entry: `HH:MM-HH:MM  project  description`.
  The project field has no spaces; everything after it is the description.
- Blank lines are ignored.

## Validation

Parsing fails, with every problem reported at once (not just the first),
when:

- a date header isn't a real calendar date (leap years included)
- a time isn't a valid `HH:MM`, or the range's end isn't after its start
- an entry appears before any `# YYYY-MM-DD` header
- two entries on the same day overlap

## Usage

```
cargo run -- path/to/september.timesheet
```

Or pipe input in instead of naming a file:

```
cat path/to/september.timesheet | cargo run -- --stdin
```

```
2026-09-15
  09:00-12:30  acme/website  build the login page
  13:15-17:00  acme/website  code review with the client
  09:00-10:00  internal      standup
  total: 7h15m

2026-09-16
  09:00-11:00  acme/website  fix the layout bug on mobile
  total: 2h00m

grand total: 9h15m
```

Add `--json` for the machine-readable form:

```
cargo run -- path/to/september.timesheet --json
```

```json
{"days":[{"date":"2026-09-15","total_minutes":435,"entries":[{"start":"09:00","end":"12:30","project":"acme/website","description":"build the login page","duration_minutes":210},{"start":"13:15","end":"17:00","project":"acme/website","description":"code review with the client","duration_minutes":225},{"start":"09:00","end":"10:00","project":"internal","description":"standup","duration_minutes":60}]}],"grand_total_minutes":435}
```

When the file has errors, `--json` reports those as JSON too, instead of
switching to plain text:

```json
{"errors":[{"line":4,"message":"end time 09:00 is not after start time 09:00"}]}
```

Without `--json`, errors go to stderr as `line N: message`, and the process
exits with status 1.

## Building

No third-party dependencies, so plain `cargo build` or `cargo run` works
with only the standard library.

## Status

Early skeleton: no per-project rollups yet, no week/month summaries, no
`--strict` mode for unpaid gaps between entries. See the code for the exact
grammar until this README grows a formal spec.
