# Deadline
The idea is to have a CLI application through which you may
- [x] Add deadlines
- [x] Add the command to your `bashrc`/whatever equivalent script so that it pops up every time you open your terminal
- [x] Optionally autoclear completed deadlines with `-c` flag when creating the task
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
```

# Contribute
Ideas/recommendations/bugfix requests are all welcome, contact me via <nandagopalnmenon@icloud.com> or submit an issue. Submit a PR if you're trying to contribute and I'll check it out
