use std::path::PathBuf;

use anyhow::{bail, Context};

use crate::viewport::{INITIAL_WINDOW_HEIGHT, INITIAL_WINDOW_WIDTH};

pub const HELP: &str = "usage: sim [options]\n\
\n\
options:\n\
  --seed S             seed world generation and simulation\n\
  --start STATE        start at main-menu or playing\n\
  --ticks N            advance N fixed simulation ticks before capture\n\
  --screenshot PATH    save one rendered frame as a PNG and exit\n\
  --width PX           capture width in physical pixels (default: 800)\n\
  --height PX          capture height in physical pixels (default: 600)\n\
  -h, --help           print this help\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartState {
    MainMenu,
    Playing,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CliOptions {
    pub seed: Option<u64>,
    pub start: Option<StartState>,
    pub ticks: u64,
    pub screenshot: Option<PathBuf>,
    pub width: u32,
    pub height: u32,
    pub help: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            seed: None,
            start: None,
            ticks: 0,
            screenshot: None,
            width: INITIAL_WINDOW_WIDTH,
            height: INITIAL_WINDOW_HEIGHT,
            help: false,
        }
    }
}

impl CliOptions {
    pub fn parse(args: impl IntoIterator<Item = String>) -> anyhow::Result<Self> {
        let mut options = Self::default();
        let mut args = args.into_iter();

        while let Some(argument) = args.next() {
            match argument.as_str() {
                "-h" | "--help" => options.help = true,
                "--seed" => {
                    options.seed = Some(parse_value(&mut args, "--seed")?);
                }
                "--start" => {
                    let value = next_value(&mut args, "--start")?;
                    options.start = Some(match value.as_str() {
                        "main-menu" => StartState::MainMenu,
                        "playing" => StartState::Playing,
                        _ => {
                            bail!("invalid --start value {value:?}; expected main-menu or playing")
                        }
                    });
                }
                "--ticks" => options.ticks = parse_value(&mut args, "--ticks")?,
                "--screenshot" => {
                    options.screenshot =
                        Some(PathBuf::from(next_value(&mut args, "--screenshot")?));
                }
                "--width" => options.width = parse_value(&mut args, "--width")?,
                "--height" => options.height = parse_value(&mut args, "--height")?,
                _ => bail!("unknown argument {argument:?}\n\n{HELP}"),
            }
        }

        if options.width == 0 || options.height == 0 {
            bail!("capture dimensions must be greater than zero");
        }
        if options.screenshot.is_none() && options.ticks != 0 {
            bail!("--ticks requires --screenshot");
        }
        if options.start == Some(StartState::MainMenu) && options.ticks != 0 {
            bail!("--ticks cannot be used with --start main-menu");
        }

        Ok(options)
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, option: &str) -> anyhow::Result<String> {
    args.next()
        .with_context(|| format!("{option} requires a value"))
}

fn parse_value<T>(args: &mut impl Iterator<Item = String>, option: &str) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value = next_value(args, option)?;
    value
        .parse()
        .with_context(|| format!("invalid value {value:?} for {option}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> anyhow::Result<CliOptions> {
        CliOptions::parse(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn parses_capture_options() {
        let options = parse(&[
            "--seed",
            "42",
            "--start",
            "playing",
            "--ticks",
            "300",
            "--screenshot",
            "capture.png",
            "--width",
            "1024",
            "--height",
            "768",
        ])
        .unwrap();

        assert_eq!(options.seed, Some(42));
        assert_eq!(options.start, Some(StartState::Playing));
        assert_eq!(options.ticks, 300);
        assert_eq!(options.screenshot, Some(PathBuf::from("capture.png")));
        assert_eq!(options.width, 1024);
        assert_eq!(options.height, 768);
    }

    #[test]
    fn rejects_ticks_without_screenshot() {
        let error = parse(&["--ticks", "1"]).unwrap_err();
        assert!(error.to_string().contains("--ticks requires --screenshot"));
    }

    #[test]
    fn rejects_ticks_for_main_menu_capture() {
        let error = parse(&[
            "--start",
            "main-menu",
            "--ticks",
            "1",
            "--screenshot",
            "capture.png",
        ])
        .unwrap_err();
        assert!(error.to_string().contains("--start main-menu"));
    }
}
