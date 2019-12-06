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


# Usage

```
USAGE:
    gnuplotters [FLAGS] [OPTIONS] --input <INPUTS>...

FLAGS:
    -f, --file       output only script file. (without figure file)
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --input <INPUTS>...              input file names
    -o, --output <OUTPUT>                output file name
    -a, --axis <axes>...                 axes in input file. (ex. x_a:y_a,x_b:y_b, ...) [default: 1:2]
    -c, --color <colors>...              plot color in each axes. [default: black]
        --fontsize <fontsize>            fontsize in title and label etc.. [default: 12]
    -l, --linetype <linetypes>...        line type in each series. [default: 1]
    -s, --seriestype <seriestypes>...    series type in each series. [default: l]  [possible values: l, p, y]
    -t, --title <titles>...              title in each series. [default: ]
    -w, --width <widths>...              each line width [default: 1]
    -x, --xlabel <xlabel>                xlabel name [default: ]
    -y, --ylabel <ylabel>                ylabel name [default: ]
```

## `-a --axis` 

specify x-axis and y-axis, expressed as `x:y`.

in following example, 1st column is used as x-axis, and 2nd column is used as y-axis.

```bash
$ gnuplotters -a 1:2 -i data.csv
```

we can specify multiple plots in a file.  
in next example, two lines are drawn. (1st line uses 1st and 2nd column, and 2nd line uses 3rd and 4th column.)

```bash
$ gnuplotters -a 1:2,3:4 -i data.csv
```
