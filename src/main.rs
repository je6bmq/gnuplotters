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
    x_label: String,
    y_label: String,
}
#[derive(Debug)]
struct Series {
    data_file: String,
    title: Option<String>,
    axes: (u32, u32),
    y_errorbar: Option<u32>,
    s_type: SeriesType,
    l_size: f32,
    color: Color,
    l_type: u32,
}

#[derive(Debug,PartialEq,Clone)]
enum SeriesType {
    Line,
    Point,
    YERRORBAR,
}
#[derive(Debug,Clone,PartialEq)]
enum Color {
    Name(String),
    Code(String),
}

impl SeriesType {
    fn series_specifier(&self, size: f32) -> String {
        match *self {
            SeriesType::Line => format!("line lw {}", size),
            SeriesType::Point => format!("point ps {}", size),
            SeriesType::YERRORBAR => format!("yerrorbars ps {}", size),
        }
    }
    fn linetype_specifier(&self, linetype: u32) -> String {
        match *self {
            SeriesType::Line => format!("dt {}", linetype),
            SeriesType::Point | SeriesType::YERRORBAR => format!("pt {}", linetype),
        }
    }
}
impl PlotScript {
    fn new() -> PlotScript {
        PlotScript {
            terminal: "pdf".to_string(),
            font: "Times New Roman, 24".to_string(),
            delimiter: r"\t".to_string(),
            legend_position: "above".to_string(),
            plot: Vec::new(),
            x_label: "".to_string(),
            y_label: "".to_string(),
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
    fn x_label(&mut self, label: String) -> &mut PlotScript {
        self.x_label = label;
        self
    }
    fn y_label(&mut self, label: String) -> &mut PlotScript {
        self.y_label = label;
        self
    }
    fn plot(&mut self, series: Series) -> &mut PlotScript {
        self.plot.push(series);
        self
    }
    fn finalize(&self, output: String) -> String {
        let is_no_title = self.plot.iter().peekable().all(|ref p| p.title.is_none());
        let legend_config = if is_no_title {String::from("")} else {format!("set key {}\nset key box lt 1 lc \"black\"\n",self.legend_position)};
        let config = format!("set terminal {} enhanced font \"{}\"\nset datafile separator \
                              \"{}\"\n{}set xlabel \
                              \"{}\"\nset ylabel \"{}\"\nset output {}",
                             self.terminal,
                             self.font,
                             self.delimiter,
                             legend_config,
                             self.x_label,
                             self.y_label,
                             if cfg!(target_os = "windows") {
                                 "\"nul\""
                             } else {
                                 "\"/dev/null\""
                             });
        format!("{}{}",
                config,
                self.plot
                    .split_first()
                    .map(|(first, cons)| {
                format!("\n\nplot {}\n{}set output \"{}\"\nreplot",
                        first.to_script(),
                        cons.iter()
                            .map(|plt| format!("replot {}\n", plt.to_script()))
                            .collect::<Vec<_>>()
                            .join(""),
                        path_split_escaper(output))
            })
                    .unwrap_or("".to_string()))


    }
}
impl Series {
    fn new(file: String,
           name: String,
           ax: (u32, u32),
           ye: Option<u32>,
           typ: SeriesType,
           size: f32,
           cl: Color,
           lt: u32)
           -> Self {
        Series {
            data_file: path_split_escaper(file),
            title: if name.len() == 0 { None } else { Some(name) },
            axes: ax,
            y_errorbar: ye,
            s_type: typ,
            l_size: size,
            color: cl,
            l_type: lt,
        }
    }
    fn to_script(&self) -> String {
        let (x, y) = self.axes;
        format!("\"{}\" using {} {} with {} lc {} {}",
                self.data_file,
                format!("{}:{}{}",
                        x,
                        y,
                        if self.s_type == SeriesType::YERRORBAR {
                            format!(":{}", self.y_errorbar.unwrap_or(y + 1))
                        } else {
                            "".to_string()
                        }),
                self.title
                    .clone()
                    .map(|pat| format!("title \"{}\"", pat))
                    .unwrap_or(format!("notitle")),
                self.s_type.series_specifier(self.l_size),
                self.color.clone().specifier(),
                self.s_type.linetype_specifier(self.l_type))
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
    fn specifier(self) -> String {
        match self {
            Color::Name(expr) => format!("\"{}\"", expr),
            Color::Code(expr) => format!("rgb \"#{}\"", expr),
        }
    }
}
fn axes_validator(arg: String) -> Result<(), String> {
    let axes_regex = Regex::new(r"^[1-9]\d*:[1-9]\d*(:[1-9]\d*)?$").unwrap();
    if arg.split(",").all(|s| axes_regex.is_match(s)) {
        Ok(())
    } else {
        Err(String::from("axes format is invalid .."))
    }
}
fn colors_validator(arg: String) -> Result<(), String> {
    let arg_list: Vec<_> = arg.split(",").collect();
    let rgb_regex = Regex::new(r"^([a-f]|[A-F]|[0-9]){6}$").unwrap();
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
fn widths_validator(arg: String) -> Result<(), String> {
    let width_regex = Regex::new(r"^([1-9][0-9]*|0)(.[0-9]+)?$").unwrap(); // match only floating value
    if arg.split(",").all(|s| width_regex.is_match(s)) {
        Ok(())
    } else {
        Err(String::from("width value is not number."))
    }
}
fn linetypes_validator(arg: String) -> Result<(), String> {
    let linetype_regex = Regex::new(r"^\d+$").unwrap();
    if arg.split(",").all(|s| linetype_regex.is_match(s)) {
        Ok(())
    } else {
        Err(String::from("linetype value is invalid (not positive number)."))
    }
}
fn path_split_escaper(s: String) -> String {
    let separator_regex =
        Regex::new(regex::escape(path::MAIN_SEPARATOR.to_string().as_str()).as_str()).unwrap();
    separator_regex.replace_all(s.as_str(), r"/").to_string()
}
fn main() {
    let app = app_from_crate!()
        .arg(Arg::with_name("INPUTS")
            .help("input file names")
            .required(true)
            .multiple(true)
            .short("i")
            .long("input")
            .takes_value(true))
        .arg(Arg::with_name("OUTPUT")
            .help("output file name")
            .required(false)
            .short("output")
            .long("output")
            .takes_value(true))
        .arg(Arg::with_name("xlabel")
            .help("xlabel name")
            .required(false)
            .short("x")
            .long("xlabel")
            .takes_value(true)
            .multiple(false)
            .default_value(""))
        .arg(Arg::with_name("ylabel")
            .help("ylabel name")
            .required(false)
            .short("y")
            .long("ylabel")
            .takes_value(true)
            .multiple(false)
            .default_value(""))
        .arg(Arg::with_name("axes")
            .help("axes in input file. (ex. x_a:y_a,x_b:y_b, ...)")
            .short("a")
            .long("axis")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(false)
            .default_value("1:2")
            .validator(axes_validator))
        .arg(Arg::with_name("titles")
            .help("title in each series.")
            .short("t")
            .long("title")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .default_value(""))
        .arg(Arg::with_name("colors")
            .help("plot color in each axes.")
            .short("c")
            .long("color")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .default_value("black")
            .validator(colors_validator))
        .arg(Arg::with_name("seriestypes")
            .help("series type in each series.")
            .short("s")
            .long("seriestype")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .possible_values(&["l", "p", "y"])
            .default_value("l"))
        .arg(Arg::with_name("widths")
            .help("each line width")
            .short("w")
            .long("width")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .default_value("1")
            .validator(widths_validator))
        .arg(Arg::with_name("linetypes")
            .help("line type in each series.")
            .short("l")
            .long("linetype")
            .takes_value(true)
            .multiple(true)
            .require_delimiter(true)
            .default_value("1")
            .validator(linetypes_validator))
        .arg(Arg::with_name("file")
            .help("output only script file. (without figure file)")
            .short("f")
            .long("file")
            .takes_value(false)
            .multiple(false))
        .arg(Arg::with_name("fontsize")
            .help("fontsize in title and label etc..")
            .long("fontsize")
            .takes_value(true)
            .default_value(if cfg!(target_os = "macos") {
                "24"
            } else {
                "12"
            })
            .validator(widths_validator));

    let args = app.get_matches();
    let data_files: Vec<&str> = args.values_of("INPUTS").unwrap().collect();
    let is_script = args.is_present("file");
    let output_file = args.value_of("OUTPUT")
        .map(|out| out.to_string())
        .unwrap_or(Regex::new(r"\.[^.]*$").unwrap().replace(data_files[0], ".pdf").into_owned());
    let xlabel = args.value_of("xlabel").unwrap().to_string();
    let ylabel = args.value_of("ylabel").unwrap().to_string();
    let axes = args.values_of("axes")
        .unwrap()
        .map(|it| {
            it.split(",")
                .map(|s| s.split(":").map(|k| k.parse::<u32>().unwrap()).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let titles = args.values_of("titles")
        .unwrap()
        .map(|t| t.to_string())
        .collect::<Vec<_>>();
    let colors = args.values_of("colors").unwrap().collect::<Vec<_>>();
    let series_types = args.values_of("seriestypes")
        .unwrap()
        .map(|it| match it {
            "l" => SeriesType::Line,
            "p" => SeriesType::Point,
            "y" => SeriesType::YERRORBAR,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    let widths = args.values_of("widths")
        .unwrap()
        .map(|w| w.parse::<f32>().unwrap())
        .collect::<Vec<_>>();
    let linetypes = args.values_of("linetypes")
        .unwrap()
        .map(|w| w.parse::<u32>().unwrap())
        .collect::<Vec<_>>();
    let fontsize = args.value_of("fontsize").unwrap().to_string();
    let script = axes.iter()
        .zip(data_files.iter())
        .map(|(ax, &data)| std::iter::repeat(data).zip(ax.into_iter()))
        .flat_map(|it| it)
        .zip(titles.into_iter().chain((0..).map(|_| "".to_string())))
        .zip(series_types.into_iter().cycle())
        .zip(widths.into_iter().cycle())
        .zip(colors.into_iter().cycle())
        .zip(linetypes.into_iter().cycle())
        .map(|((((((d, a), t), s), w), c), lt)| {
            Series::new(d.to_string(),
                        t,
                        (a[0], a[1]), //(x,y)
                        a.get(2).map(|&y_error| y_error),
                        s,
                        w,
                        Color::new(c.to_string()),
                        lt)
        })
        .fold(PlotScript::new()
                  .delimiter(",".to_string())
                  .font(format!("Times New Roman, {}", fontsize))
                  .x_label(xlabel)
                  .y_label(ylabel),
              |plt, ser| plt.plot(ser))
        .finalize(output_file.clone());

    if is_script {
        let script_file =
            Regex::new(r"\.[^.]*$").unwrap().replace(output_file.as_str(), ".gplot").into_owned(); // replacement of extension(suffix) in filename
        File::create(script_file).unwrap().write_all(script.as_bytes()).unwrap();

    } else {

        let temp_file = Temp::new_file_in(&(path::Path::new(data_files[0]).parent().unwrap()))
            .unwrap();
        let tmp_path = temp_file.as_ref().to_path_buf();
        let tmp_path = tmp_path.as_path().to_str().unwrap();
        let written = File::create(temp_file.as_ref()).unwrap().write_all(script.as_bytes());
        assert!(written.is_ok());
        let tmp_path = Regex::new(regex::escape(path::MAIN_SEPARATOR.to_string().as_str())
                .as_str())
            .unwrap()
            .replace_all(tmp_path, r"/");

        let _ = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .arg("/C")
                    .arg("gnuplot")
                    .arg("-e")
                    .arg(format!("load \"{}\"",
                                 regex::escape(tmp_path.to_string().as_str()).as_str())
                        .as_str())
                    .output()
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(format!("gnuplot -e \"load \\\"{}\\\"\"",
                                 regex::escape(tmp_path.to_string().as_str()).as_str())
                        .as_str())
                    .output()
            }
            .expect("failed to execute gnuplot. ");
    }
}

#[test]
fn axes_validatior_test() {
    assert!(axes_validator("1:2".to_string()).is_ok());
    assert!(axes_validator("1:2:3,1:2".to_string()).is_ok());
    assert!(axes_validator("1:2,3".to_string()).is_err());
    assert!(axes_validator("1:2:".to_string()).is_err());
}
#[test]
fn colors_validator_test() {
    assert!(colors_validator("red,f8Ab05".to_string()).is_ok());
    assert!(colors_validator("lered,aaaagg".to_string()).is_err());
    assert!(colors_validator("#ABCDEF".to_string()).is_err());
}
#[test]
fn widths_validator_test() {
    assert!(widths_validator("1.00".to_string()).is_ok());
    assert!(widths_validator("1".to_string()).is_ok());
    assert!(widths_validator("1.0f".to_string()).is_err());
}
#[test]
fn linetypes_validator_test() {
    assert!(linetypes_validator("1,10,50".to_string()).is_ok());
    assert!(linetypes_validator("10,-5,50".to_string()).is_err());
}
#[test]
fn path_split_escaper_test() {
    assert_eq!(path_split_escaper("C:\\test\\hoge.csv".to_string()),
               "C:/test/hoge.csv".to_string());
}
#[test]
fn line_specifier_test() {
    assert_eq!(SeriesType::Line.series_specifier(1.0),
               "line lw 1".to_string());
    assert_eq!(SeriesType::Point.series_specifier(1.0),
               "point ps 1".to_string());
    assert_eq!(SeriesType::YERRORBAR.series_specifier(1.0),
               "yerrorbars ps 1".to_string());
    assert_eq!(SeriesType::Line.series_specifier(1.95),
               "line lw 1.95".to_string());
    assert_eq!(SeriesType::Point.series_specifier(1.95),
               "point ps 1.95".to_string());

    assert_eq!(SeriesType::YERRORBAR.series_specifier(1.95),
               "yerrorbars ps 1.95".to_string());
}
#[test]
fn linetype_specifier_test() {
    assert_eq!(SeriesType::Line.linetype_specifier(1), "dt 1".to_string());
    assert_eq!(SeriesType::Point.linetype_specifier(1), "pt 1".to_string());
    assert_eq!(SeriesType::YERRORBAR.linetype_specifier(1),
               "pt 1".to_string());
    assert_eq!(SeriesType::Line.linetype_specifier(100),
               "dt 100".to_string());
    assert_eq!(SeriesType::Point.linetype_specifier(100),
               "pt 100".to_string());
    assert_eq!(SeriesType::YERRORBAR.linetype_specifier(100),
               "pt 100".to_string());
}
#[test]
fn color_new_test() {
    assert_eq!(Color::new("blue".to_string()),
               Color::Name("blue".to_string()));
    assert_eq!(Color::new("99ab55".to_string()),
               Color::Code("99ab55".to_string()));
}
#[test]
fn color_specifier_test() {
    let red = Color::Name("red".to_string());
    let blue_code = Color::Code("0000FF".to_string());
    assert_eq!(red.specifier(), "\"red\"".to_string());
    assert_eq!(blue_code.specifier(), "rgb \"#0000FF\"".to_string());
}
#[test]
fn series_to_plot_test() {
    let series = Series::new("test.csv".to_string(),
                             "".to_string(),
                             (1, 2),
                             None,
                             SeriesType::Line,
                             1.5,
                             Color::new("red".to_string()),
                             1);
    assert_eq!(series.to_script(),
               "\"test.csv\" using 1:2 notitle with line lw 1.5 lc \"red\" dt 1".to_string());
    let series = Series::new("hoge.csv".to_string(),
                             "".to_string(),
                             (10, 5),
                             None,
                             SeriesType::Point,
                             1.0,
                             Color::new("afBF55".to_string()),
                             15);
    assert_eq!(series.to_script(),
               "\"hoge.csv\" using 10:5 notitle with point ps 1 lc rgb \"#afBF55\" pt 15"
                   .to_string());
    let series = Series::new("hoge.csv".to_string(),
                             "".to_string(),
                             (10, 5),
                             None,
                             SeriesType::YERRORBAR,
                             1.0,
                             Color::new("afBF55".to_string()),
                             15);
    assert_eq!(series.to_script(),
               "\"hoge.csv\" using 10:5:6 notitle with yerrorbars ps 1 lc rgb \"#afBF55\" pt 15"
                   .to_string());
    let series = Series::new("hoge.csv".to_string(),
                             "".to_string(),
                             (10, 5),
                             Some(11),
                             SeriesType::YERRORBAR,
                             1.0,
                             Color::new("afBF55".to_string()),
                             15);
    assert_eq!(series.to_script(),
               "\"hoge.csv\" using 10:5:11 notitle with yerrorbars ps 1 lc rgb \"#afBF55\" pt 15"
                   .to_string());
}
#[test]
fn finalize_without_series_test() {
    let script = PlotScript::new();
    let output = String::from("hoge.pdf");
    assert_eq!(script.finalize(output.clone()),
               format!("set terminal pdf enhanced font \"Times New Roman, 24\"\nset datafile \
                        separator \"\\t\"\nset \
                        xlabel \"\"\nset ylabel \"\"\nset output {}",
                       if cfg!(target_os = "windows") {
                           "\"nul\""
                       } else {
                           "\"/dev/null\""
                       }));
}
#[test]
fn finalize_with_series_test() {
    let mut script = PlotScript::new();
    let output = String::from("hoge.pdf");
    let series = Series::new("test.csv".to_string(),
                             "".to_string(),
                             (1, 2),
                             None,
                             SeriesType::Line,
                             1.5,
                             Color::new("red".to_string()),
                             1);
    script.plot(series);
    assert_eq!(script.finalize(output.clone()),
               format!("set terminal pdf enhanced font \"Times New Roman, 24\"\nset datafile \
                        separator \"\\t\"\nset \
                        xlabel \"\"\nset ylabel \"\"\nset output {}\n\nplot \"test.csv\" using \
                        1:2 notitle with line lw 1.5 lc \"red\" dt 1\nset output \"{}\"\nreplot",
                       if cfg!(target_os = "windows") {
                           "\"nul\""
                       } else {
                           "\"/dev/null\""
                       },
                       output.clone()));
    let series2 = Series::new("hoge.csv".to_string(),
                              "".to_string(),
                              (10, 5),
                              None,
                              SeriesType::Point,
                              1.0,
                              Color::new("afBF55".to_string()),
                              15);
    script.plot(series2);
    assert_eq!(script.finalize(output.clone()),
               format!("set terminal pdf enhanced font \"Times New Roman, 24\"\nset datafile \
                        separator \"\\t\"\nset \
                        xlabel \"\"\nset ylabel \"\"\nset output {}\n\nplot \"test.csv\" using \
                        1:2 notitle with line lw 1.5 lc \"red\" dt 1\nreplot \"hoge.csv\" using \
                        10:5 notitle with point ps 1 lc rgb \"#afBF55\" pt 15\nset output \
                        \"{}\"\nreplot",
                       if cfg!(target_os = "windows") {
                           "\"nul\""
                       } else {
                           "\"/dev/null\""
                       },
                       output.clone()));
}
#[test]
fn finalize_custom_script_test() {
    let mut script = PlotScript::new();
    script.delimiter(",".to_string())
        .font("Century".to_string())
        .legend(vec!["left".to_string(), "top".to_string()])
        .x_label("Axis X".to_string())
        .y_label("Axis Y".to_string());
    let output = String::from("hoge.pdf");
    let series = Series::new("test.csv".to_string(),
                             "test".to_string(),
                             (1, 2),
                             Some(4),
                             SeriesType::YERRORBAR,
                             1.5,
                             Color::new("red".to_string()),
                             1);
    script.plot(series);
    assert_eq!(script.finalize(output.clone()),
               format!("set terminal pdf enhanced font \"Century\"\nset datafile separator \
                        \",\"\nset key left top\nset key box lt 1 lc \"black\"\nset xlabel \
                        \"Axis X\"\nset ylabel \"Axis Y\"\nset output {}\n\nplot \"test.csv\" \
                        using 1:2:4 title \"test\" with yerrorbars ps 1.5 lc \"red\" pt 1\nset \
                        output \"{}\"\nreplot",
                       if cfg!(target_os = "windows") {
                           "\"nul\""
                       } else {
                           "\"/dev/null\""
                       },
                       output.clone()));
}
