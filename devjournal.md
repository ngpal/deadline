# 05-03-2026

- [x] add deadlines
- [x] add command
- [x] view command
- [x] store the deadlines in a file
  - [x] view storage file path
- [x] autoclear

# 07-03-2026

- [x] hashing the tasks for id

# 08-03-2026

- [x] deleting tasks
- [x] indication while printing for autoclear tasks
- [x] completing tasks behind deadline
  - [x] cleaner printing for completed tasks
- [x] `view` command options
  - [x] default shows due and overdue tasks
  - [x] `--reverse` should show the tasks in reverse order
  - [x] `--completed` should show completed tasks only
  - [x] `--overdue` should show overdue tasks only
  - [x] `--no-hash` hide hashes
  - [x] `--no-title` hide title
  - [x] `--all` should show all the tasks, overdue at the top, completed at the end

- [x] allow hashes to be searched by prefix if only one match is available
- [x] allow Xd format for deadlines for X number of days since today
- [x] just remove the title by default its annoying i dont like it anymore
- [x] update readme with usage
- [x] `view --no-flags` to hide the autoclear flags

# 09-03-2026

- [x] `view --lines n` to show top n number of results
- [x] editing tasks
  - [x] pushing deadlines
  - [x] changing autostrike status
  - [x] changing complete status

# 22-07-2026

- [x] macOS calendar integration
  - [x] Swift helper binary (calendar-reader.swift) using EventKit
  - [x] `deadline calendar` subcommand to view upcoming events (alias: `cal`)
  - [x] `--calendar`/`-C` flag on `deadline view` to merge events with tasks
  - [x] deterministic hash for calendar events (title + start date)
  - [x] strike/unstrike support for calendar events via `struck_events.json`
  - [x] calendar name display (e.g. `[Work]`, `[Personal]`)
  - [x] `--name` filter for calendar and view commands
  - [x] `--completed` flag on calendar subcommand
  - [x] graceful fallback on non-macOS with warning
  - [x] local timezone handling for event dates
  - [x] version bump to 0.7.1
- [x] multiple calendar name filtering
  - [x] `--name` accepts comma-separated values (e.g. `--name Work,Personal`)
  - [x] version bump to 0.7.2
- [x] config file system
  - [x] `Config`, `ColorConfig`, `CalendarConfig` structs with serde defaults
  - [x] `deadline config show|set|reset|path` subcommand (alias: `cfg`)
  - [x] configurable colors for overdue/warning/safe/completed/hash
  - [x] configurable calendar defaults (default_days, default_names)
  - [x] `OnceLock` singleton for config access
  - [x] version bump to 0.8.0

# Future

- [ ] better parsing allowing day of the week for deadlines
- [ ] reccuring tasks
  - recurring tasks without autoclear show all previous tasks
  - recurring tasks just create a new task when their deadlines are hit/struck out
  - [ ] reccuringness can be edited
  - [ ] relative (same date every month/year or so) and absolute (n days)
