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

# 08-04-2026
- [ ] reccuring tasks
  - recurring tasks without autoclear show all previous tasks
  - recurring tasks just create a new task when their deadlines are hit/struck out
  - [ ] reccuringness can be edited
  - [ ] relative (same date every month/year or so) and absolute (n days)

- [ ] config file
  - [ ] config editing commands
  - [ ] config for colours
