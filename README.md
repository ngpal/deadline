# Deadline
The idea is to have a CLI application through which you may
- [x] Add deadlines
- [x] Add the command to your `bashrc`/whatever equivalent script so that it pops up every time you open your terminal
- [x] Optionally autostrike completed deadlines with `-s` flag when creating the task
- [x] Read upcoming events from your macOS calendar
- [ ] Progress percentages you can update yourself or autoupdate with time
- [x] Customization on colours

# Installation
Clone the repository locally and install with cargo
```sh
cargo install --git https://github.com/ngpal/deadline

# OR

git clone https://github.com/ngpal/deadline <path>
cargo install --path <path-to-deadline>
```
(Optional) Add the following lines to your  `.bashrc` or respective "run commands" file equivalent
```sh
# now you can use `dl` instead of `deadline`
alias dl='deadline'

# prints all the tasks every time you open your terminal
dl view
````

# Usage

`deadline` lets you track deadlines from your terminal.

## Add tasks

```sh
deadline add "Finish assignment" 2026-03-10
````

You can also specify deadlines relative to today:

```sh
deadline add "Finish assignment" 3d
```

Or by the next occurrence of a weekday (full name or abbreviation):

```sh
deadline add "Pay rent" friday
deadline add "Standup" mon
```

If the day you enter is today, the deadline is scheduled for next week.

Automatically strike a task once the deadline passes:

```sh
deadline add "Submit report" 2026-03-10 -s
```

## View tasks

```sh
deadline view
```

Useful filters:


```sh
deadline view --completed   # completed tasks
deadline view --overdue     # overdue tasks
deadline view --all         # all tasks
deadline view -r            # reverse order
deadline view --no-hash     # hide task hashes
deadline view -t "Title"    # print a custom title
```

## Colors

Color is on by default when stdout is a terminal and off when piped to a file or another program. Force it with the `--color` flag:

```sh
deadline --color always view | less   # keep colors when piping
deadline --color=always view          # same, with explicit value
deadline --color never view           # disable colors in a terminal
deadline --color auto view            # back to auto-detection
```

`--color` is a global flag, so it works before or after the subcommand (`deadline view --color=never`). Bare `--color` (no value) means `always`. Colors remain configurable via `config set colors.*`.

## Complete / reopen tasks

Mark a task as completed:

```sh
deadline strike <hash>
```

Undo completion:

```sh
deadline unstrike <hash>
```

Hashes can be unique prefixes of the task ID.

## Delete tasks

```sh
deadline del <hash>
```

Skip confirmation:

```sh
deadline del <hash> -f
```

## Show data file location

```sh
deadline path
```

## Calendar integration (macOS only)

Deadline can read events from your macOS calendar. First, compile the Swift helper:

```sh
swiftc calendar-reader.swift -o calendar-reader -framework EventKit
```

View upcoming calendar events:

```sh
deadline calendar                   # next 14 days (alias: cal)
deadline cal -d 7                   # next 7 days
deadline cal --name Work            # filter by calendar name
deadline cal --name Work,Personal   # filter by multiple calendars
deadline cal --completed            # show struck events too
```

Strike/unstrike calendar events using the same commands as tasks:

```sh
deadline strike <hash>        # strike a calendar event or task
deadline unstrike <hash>      # unstrike a calendar event or task
```

Calendar events display their calendar name (e.g., `[Work]`, `[Personal]`) and can be filtered by it.

Merge calendar events into the task view:

```sh
deadline view -C                  # tasks + calendar events
deadline view -C --cal-days 7     # tasks + next 7 days of events
deadline view -C --name Work      # tasks + events from "Work" calendar
deadline view -C --name Work,Personal  # multiple calendars
```

Calendar events show with a hex hash (same format as tasks) and a `(calendar)` suffix. Struck events are shown in blue with strikethrough, same as tasks.

> **Note:** Calendar integration requires macOS and the `calendar-reader` binary in your working directory. On other platforms, these commands print a warning and show no events. Calendar event strike state is stored locally in `struck_events.json`.

## Configuration

Deadline stores its configuration in `config.json` alongside the task data. Use the `config` subcommand (alias: `cfg`) to manage it.

```sh
deadline config show                        # display current config
deadline config set colors.overdue red      # set overdue color
deadline config set calendar.default_days 7 # default calendar look-ahead
deadline config set calendar.default_names Work,Personal  # default calendar filter
deadline config reset                       # reset to defaults
deadline config path                        # show config file path
```

### Available config keys

| Key | Default | Description |
|-----|---------|-------------|
| `colors.overdue` | `red` | Color for overdue tasks/events |
| `colors.warning` | `yellow` | Color for tasks due within 5 days |
| `colors.safe` | `green` | Color for tasks due later |
| `colors.completed` | `blue` | Color for struck/completed items |
| `colors.hash` | `cyan` | Color for task/event hashes |
| `calendar.default_days` | `14` | Default look-ahead for calendar commands |
| `calendar.default_names` | *(empty)* | Default calendar name filter |

# Contribute
Ideas/recommendations/bugfix requests are all welcome, contact me via <nandagopalnmenon@icloud.com> or submit an issue. Submit a PR if you're trying to contribute and I'll check it out
