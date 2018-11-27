# gnuplotters
This project is "simple CUI gnuplot executer". It is written by Rust.

# Install

```bash
$ curl -sSf https://rustup.sh.rs | sh
$ cargo install --git https://github.com/je6bmq/gnuplotters
$ gnuplotters --help
```

# Example
If you use 1st column as x-axis and 2nd column as y-axis for `data.csv`, 

```bash
$ gnuplotters -a 1:2 -i data.csv
```

Then, you can obtain `data.pdf`.


## Options
To be written.
