#![allow(dead_code)]
#[macro_use]
extern crate clap;
extern crate mktemp;
extern crate regex;

use clap::Arg;
use mktemp::Temp;
use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::path;

#[derive(Debug)]
struct PlotScript {
    terminal: String,
    font: String,
    delimiter: String,
    legend_position: String,
    plot: Vec<Series>,
}
#[derive(Debug)]
struct Series {
    data_file: String,
    axes: (u32, u32),
    s_type: SeriesType,
    l_size: f32,
    color: Color,
}

#[derive(Debug,PartialEq,Clone)]
enum SeriesType {
    Line,
    Point,
}
#[derive(Debug,Clone)]
enum Color {
    Name(String),
    Code(String),
}
impl PlotScript {
    fn new() -> PlotScript {
        PlotScript {
            terminal: "pdf".to_string(),
            font: "Times New Roman".to_string(),
            delimiter: r"\t".to_string(),
            legend_position: "below".to_string(),
            plot: Vec::new(),
        }
    }
    fn terminal(&mut self, t: String) -> &mut PlotScript {
        self.terminal = t;
        self
    }
    fn font(&mut self, f: String) -> &mut PlotScript {
        self.font = f;
        self
    }
    fn delimiter(&mut self, d: String) -> &mut PlotScript {
        self.delimiter = d;
        self
    }
    fn legend(&mut self, pos: Vec<String>) -> &mut PlotScript {
        self.legend_position = pos.join(" ");
        self
    }
    fn plot(&mut self, series: Series) -> &mut PlotScript {
        self.plot.push(series);
        self
    }
    fn finalize(&self, output: String) -> String {
        let (first, cons) = self.plot.split_first().unwrap();
        let separator_regex =
            Regex::new(regex::escape(path::MAIN_SEPARATOR.to_string().as_str()).as_str()).unwrap();
        let line_detector = |t: SeriesType, s: f32| if t == SeriesType::Line {
            format!("line lw {}", s)
        } else {
            format!("point ps {}", s)
        };
        let color_detector = |c: Color| match c {
            Color::Name(expr) => format!("\"{}\"", expr),
            Color::Code(expr) => format!("rgb \"#{}\"", expr),
        };
        format!("set terminal {} enhanced font \"{}\" \nset datafile separator \"{}\" \nset key \
                 {} \n\nplot \"{}\" using {}:{} with {} lc {} \n{}\nset output \"{}\" \nreplot",
                self.terminal,
                self.font,
                self.delimiter,
                self.legend_position,
                separator_regex.replace_all(first.data_file.as_str(), r"/"),
                first.axes.0,
                first.axes.1,
                line_detector(first.s_type.clone(), first.l_size),
                color_detector(first.color.clone()),
                cons.iter()
                    .map(|plt| {
                format!(" replot \"{}\" using {}:{} with {} lc {} \n",
                        separator_regex.replace_all(plt.data_file.as_str(), r"/"),
                        plt.axes.0,
                        plt.axes.1,
                        line_detector(plt.s_type.clone(), plt.l_size),
                        color_detector(plt.color.clone()))
            })
                    .collect::<Vec<_>>()
                    .join(""),
                separator_regex.replace_all(output.as_str(), r"/"))
    }
}
impl Series {
    fn new(file: String, ax: (u32, u32), typ: SeriesType, size: f32, cl: Color) -> Self {
        Series {
            data_file: file,
            axes: ax,
            s_type: typ,
            l_size: size,
            color: cl,
        }
    }
}
impl Color {
    fn new(arg: String) -> Color {
        let rgb_regex = Regex::new(r"([a-f]|[A-F]|[0-9]){6}").unwrap();
        if rgb_regex.is_match(arg.as_str()) {
            Color::Code(arg)
        } else {
            Color::Name(arg)
        }
    }
}
fn axes_validator(arg: String) -> Result<(), String> {
    if Regex::new(r"([1-9]\d*:[1-9]\d*,)*([1-9]\d*:[1-9]\d*)").unwrap().is_match(arg.as_str()) {
        Ok(())
    } else {
        Err(String::from("axes format is invalid .."))
    }
}
fn colors_validator(arg: String) -> Result<(), String> {
    let arg_list: Vec<_> = arg.split(",").collect();
    let rgb_regex = Regex::new(r"([a-f]|[A-F]|[0-9]){6}").unwrap();
    let color_list = ["white",
                      "black",
                      "dark-grey",
                      "red",
                      "web-green",
                      "web-blue",
                      "dark-magenta",
                      "dark-cyan",
                      "dark-orange",
                      "radk-yellow",
                      "royalblue",
                      "goldenrod",
                      "dark-spring-green",
                      "purple",
                      "steelblue",
                      "dark-red",
                      "dark-chartreuse",
                      "orchid",
                      "aquamarine",
                      "brown",
                      "yellow",
                      "turquoise",
                      "grey0",
                      "grey10",
                      "grey20",
                      "grey30",
                      "grey40",
                      "grey50",
                      "grey60",
                      "grey70",
                      "grey",
                      "grey80",
                      "grey90",
                      "grey100",
                      "light-red",
                      "light-green",
                      "light-blue",
                      "light-magenta",
                      "light-cyan",
                      "light-goldenrod",
                      "light-pink",
                      "light-turquoise",
                      "gold",
                      "green",
                      "dark-green",
                      "spring-green",
                      "forest-green",
                      "sea-green",
                      "blue",
                      "dark-blue",
                      "midnight-blue",
                      "navy",
                      "midium-blue",
                      "skyblue",
                      "cyan",
                      "magenta",
                      "dark-turquoise",
                      "dark-pink",
                      "coral",
                      "light-coral",
                      "orange-red",
                      "salmon",
                      "dark-salmon",
                      "khaki",
                      "dark-khaki",
                      "dark-goldenrod",
                      "beige",
                      "olive",
                      "orange",
                      "violet",
                      "dark-violet",
                      "plum",
                      "dark-plum",
                      "dark-olivegreen",
                      "orangered4",
                      "brown4",
                      "sienna4",
                      "orchid4",
                      "mediumpurple3",
                      "slateblue1",
                      "yellow4",
                      "sienna1",
                      "tan1",
                      "sandybrown",
                      "light-salmon",
                      "pink",
                      "khaki1",
                      "lemonchiffon",
                      "bisque",
                      "honeydew",
                      "slategrey",
                      "seagreen",
                      "antiquewhite",
                      "chartreuse",
                      "greenyellow",
                      "gray",
                      "light-gray",
                      "light-grey",
                      "dark-gray",
                      "slategray",
                      "gray0",
                      "gray10",
                      "gray20",
                      "gray30",
                      "gray40",
                      "gray50",
                      "gray60",
                      "gray70",
                      "gray80",
                      "gray90",
                      "gray100"];

    if arg_list.iter().all(|it| color_list.contains(it) | rgb_regex.is_match(it)) {
        Ok(())
    } else {
        Err(String::from("invalid color {name,code}"))
    }
}
fn width_validator(arg: String) -> Result<(), String> {
    let width_regex = Regex::new(r"^([1-9][0-9]*|0)(.[0-9]+)?$").unwrap(); // match only floating value
    if width_regex.is_match(arg.as_str()) {
        Ok(())
    } else {
        Err(String::from("width value is not number."))
    }
}
fn main() {
    let app = app_from_crate!()
        .arg(Arg::with_name("INPUT")
            .help("input file name")
            .required(true)
            .multiple(true)
            .index(1))
        .arg(Arg::with_name("OUTPUT").help("output file name").required(false).index(2))
        .arg(Arg::with_name("axes")
            .help("axes in input file. (ex. x_a:y_a,x_b:y_b, ...)")
            .short("a")
            .long("axes")
            .takes_value(true)
            .multiple(true) 
            .require_delimiter(true)
            .default_value("1:2")
            .validator(axes_validator))
        .arg(Arg::with_name("colors")
            .help("plot color in each axes.")
            .short("c")
            .long("colors")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .default_value("black")
            .validator(colors_validator))
        .arg(Arg::with_name("seriestypes")
            .help("linetype in each series.")
            .short("t")
            .long("seriestypes")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .possible_values(&["l", "p"])
            .default_value("l"))
        .arg(Arg::with_name("width")
            .help("each line width")
            .short("w")
            .long("width")
            .takes_value(true)
            .default_value("1")
            .validator(width_validator))
        .arg(Arg::with_name("script")
            .help("output only script file. (without figure file)")
            .short("s")
            .long("script")
            .takes_value(false));

    let args = app.get_matches();
    let data_files:Vec<&str> = args.values_of("INPUT").unwrap().collect();
    let is_script = args.occurrences_of("script") == 1;
    let output_file = if let Some(out) = args.value_of("OUTPUT") {
        out.to_string()
    } else {
        Regex::new(r"\.[^.]*$")
            .unwrap()
            .replace(data_files[0], ".pdf")
            .into_owned() // replacement of extension(suffix) in filename
    };
    let num_series = args.values_of("axes").unwrap().len();
    let axes = args.values_of("axes")
        .unwrap()
        .map(|it| {
            let s = it.split(":").map(|k| k.parse::<u32>().unwrap()).collect::<Vec<_>>();
            (s[0], s[1])
        })
        .collect::<Vec<_>>();
    let colors = args.values_of("colors").unwrap().collect::<Vec<_>>();
    let seriestypes = args.values_of("seriestypes")
        .unwrap()
        .map(|it| if it == "l" {
            SeriesType::Line
        } else {
            SeriesType::Point
        })
        .collect::<Vec<_>>();
    let width = args.value_of("width").unwrap().parse::<f32>().unwrap();
    let script = (0usize..(num_series as usize))
        .map(|i| {
            Series::new(data_files[0].to_string(),
                        axes[i],
                        seriestypes[i % seriestypes.len()].clone(),
                        width,
                        Color::new(colors[i % colors.len()].to_string()))
        })
        .fold(PlotScript::new().delimiter(",".to_string()),
              |plt, ser| plt.plot(ser))
        .finalize(output_file.clone());

    if is_script {
        let script_file = Regex::new(r"\.[^.]*$")
            .unwrap()
            .replace(output_file.as_str(), ".gplot")
            .into_owned(); // replacement of extension(suffix) in filename
        File::create(script_file).unwrap().write_all(script.as_bytes()).unwrap();

    } else {

        let temp_file = Temp::new_file_in(&(path::Path::new(data_files[0]).parent().unwrap())).unwrap();
        let tmp_path = temp_file.as_ref().to_path_buf();
        let tmp_path = tmp_path.as_path().to_str().unwrap();
        let written = File::create(temp_file.as_ref()).unwrap().write_all(script.as_bytes());
        assert!(written.is_ok());
        let tmp_path = Regex::new(regex::escape(path::MAIN_SEPARATOR.to_string().as_str())
                .as_str())
            .unwrap()
            .replace_all(tmp_path, r"/");

        let _ = Command::new(if cfg!(target_os = "windows") {
                "cmd"
            } else {
                "sh"
            })
            .arg(if cfg!(target_os = "windows") {
                "/C"
            } else {
                "-c"
            })
            .arg("gnuplot")
            .arg("-e")
            .arg(format!("load \"{}\"",
                         regex::escape(tmp_path.to_string().as_str()).as_str())
                .as_str())
            .output()
            .expect("failed to execute gnuplot. ");
    }
}

#[test]
fn validation_test() {
    assert!(axes_validator("1:2".to_string()).is_ok());
    assert!(colors_validator("red,f8Ab05".to_string()).is_ok());
}
#[test]
fn regex_test() {
    assert!(Regex::new(regex::escape(path::MAIN_SEPARATOR.to_string().as_str()).as_str()).is_ok());
}