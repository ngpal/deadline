# Deadline
The idea is to have a CLI application through which you may
- [x] Add deadlines
- [x] Add the command to your `bashrc`/whatever equivalent script so that it pops up every time you open your terminal
- [x] Optionally autostrike completed deadlines with `-s` flag when creating the task
- [ ] Progress percentages you can update yourself or autoupdate with time
- [ ] Customization on colours

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
# Contribute
Ideas/recommendations/bugfix requests are all welcome, contact me via <nandagopalnmenon@icloud.com> or submit an issue. Submit a PR if you're trying to contribute and I'll check it out
