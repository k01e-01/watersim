use std::{any::Any, io::{self, Read, Write}, iter::Peekable, path::PathBuf, str::Split, thread, time::{Duration, Instant}};

use anyhow::{bail, Result};
use log::{debug, error};
use rustyline::{history::MemHistory, Editor};
use termion::raw::IntoRawMode;

use crate::{ansi, grid::{Grid, TileKind}, physics};

#[derive(Debug)]
pub enum UserAction {
    Noop,
    Save(PathBuf),
    Load(PathBuf),
    New(u16, u16),
    Show,
    Inspect(u16, u16),
    Tick,
    Tps(u16),
    Set(u16, u16, TileKind),
    Run,
    Quit,
    Help,
}

pub fn render_world(world: &Grid, flush: bool) -> Result<()> {
    let mut stdout = io::stdout();

    for row in world.iter() {
        for tile in row.iter() {
            print!("{}", tile);
        }
        print!("\n\r");
    }

    if flush {
        stdout.flush()?;
    }

    Ok(())
}

const HELP_MESSAGE: &str = include_str!("help.txt");

pub fn show_help() {
    print!("\n{}\n", HELP_MESSAGE);
}

enum Arg<'a> {
    Str(&'a str),
    Int(u16),
    Tile(TileKind),
}

impl<'a> Arg<'a> {
    fn as_str(&self) -> Option<&'a str> {
        if let Arg::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }

    fn as_int(&self) -> Option<u16> {
        if let Arg::Int(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    fn as_tile(&self) -> Option<TileKind> {
        if let Arg::Tile(n) = self {
            Some(*n)
        } else {
            None
        }
    }
} 

fn parse_tile_arg(arg: &str) -> Result<TileKind> {
    let (tile, pressure) = match arg.split_once(':') {
        Some(tp) => tp,
        None => (arg, "1"),
    };

    Ok(match tile {
        "e" | "empty" => TileKind::Empty,
        "b" | "wall" => TileKind::Wall,
        "w" | "water" => TileKind::Water(pressure.parse()?),
        s => bail!("no such tile: {}", s),
    })
}

fn parse_args(
    args: &mut Peekable<Split<'_, char>>, 
    expected: &[(&str, &str)], 
    then: fn(Vec<Arg>) -> UserAction
) -> UserAction {
    if args.clone().collect::<Vec<_>>().len() > expected.len() {
        error!("unexpected args");
        return UserAction::Noop;
    }

    let mut arg_vec: Vec<Arg> = Vec::new();

    for (exp_arg, exp_arg_type) in expected.iter() {
        if let None = args.peek() {
            error!("missing {} arg", exp_arg);
            return UserAction::Noop;
        }

        let unparsed_arg = args.next().unwrap();
        
        let parsed_arg = match exp_arg_type {
            &"str" => Arg::Str(unparsed_arg),

            &"int" => match unparsed_arg.parse() {
                Ok(n) => Arg::Int(n),
                Err(e) => {
                    error!("failed to parse int: {}", e);
                    return UserAction::Noop
                }
            },

            &"tile" => match parse_tile_arg(unparsed_arg) {
                Ok(t) => Arg::Tile(t),
                Err(e) => {
                    error!("failed to parse tile: {}", e);
                    return UserAction::Noop
                }
            },

            &&_ => {
                error!("unknown arg type??? what???");
                return UserAction::Noop;
            },
        };

        arg_vec.push(parsed_arg);
    }

    then(arg_vec)
}

pub fn request_user_action(rl: &mut Editor<(), MemHistory>) -> Result<UserAction> {

    let input = match rl.readline(&format!(
        "{0}[{1}watersim{0}]{1} > ", 
        ansi::GREY_FG, ansi::RESET,
    )) {
        Ok(s) => s,
        Err(_) => "".to_string(),
    };

    rl.add_history_entry(input.clone())?;

    let mut args = input.trim().split(' ').peekable();

    let action = match args.next().unwrap() {
        "" | "noop" => parse_args(
            &mut args, 
            &[], 
            |_| UserAction::Noop,
        ),
 
        "s" | "save" => parse_args(
            &mut args, 
            &[
                ("path", "str"),
            ], 
            |a| UserAction::Save(
                a.get(0).unwrap().as_str().unwrap().into(),
            ),
        ),

        "l" | "load" => parse_args(
            &mut args, 
            &[
                ("path", "str"),
            ], 
            |a| UserAction::Load(
                a.get(0).unwrap().as_str().unwrap().into(),
            ),
        ),

        "n" | "new" => parse_args(
            &mut args, 
            &[
                ("width", "int"), 
                ("height", "int"),
            ], 
            |a| UserAction::New(
                a.get(0).unwrap().as_int().unwrap(), 
                a.get(1).unwrap().as_int().unwrap(),
            ),
        ),

        "v" | "show" | "view" => parse_args(
            &mut args, 
            &[], 
            |_| UserAction::Show,
        ),

        "i" | "inspect" | "debug" => parse_args(
            &mut args, 
            &[
                ("x", "int"),
                ("y", "int"),
            ], 
            |a| UserAction::Inspect(
                a.get(0).unwrap().as_int().unwrap(), 
                a.get(1).unwrap().as_int().unwrap(),
            ),
        ),

        "t" | "tick" => parse_args(
            &mut args, 
            &[], 
            |_| UserAction::Tick,
        ),

        "f" | "fps" | "tps" | "speed" => parse_args(
            &mut args, 
            &[
                ("tps", "int"),
            ], 
            |a| UserAction::Tps(
                a.get(0).unwrap().as_int().unwrap(),
            ),
        ),

        "e" | "edit" | "set" => parse_args(
            &mut args, 
            &[
                ("x", "int"),
                ("y", "int"),
                ("tile", "tile"),
            ], 
            |a| UserAction::Set(
                a.get(0).unwrap().as_int().unwrap(), 
                a.get(1).unwrap().as_int().unwrap(), 
                a.get(2).unwrap().as_tile().unwrap(),
            ),
        ),

        "r" | "run" => parse_args(
            &mut args, 
            &[], 
            |_| UserAction::Run,
        ),

        "q" | "quit" | "exit" => parse_args(
            &mut args, 
            &[], 
            |_| UserAction::Quit,
        ),

        "h" | "?" | "help" => parse_args(
            &mut args, 
            &[], 
            |_| UserAction::Help,
        ),

        s => {
            error!("unknown command: \"{}\"", s);
            UserAction::Noop
        },

    };

    debug!("user action: {:?}", &action);
    
    Ok(action)
}

pub fn run_loop(world: &mut Grid, target_tps: u16) -> Result<()> {
    let mut ticks = 0;
    let target_mspt;
    if target_tps == 0 {
        target_mspt = Duration::MAX;
    } else {
        target_mspt = Duration::from_secs_f64(1.0 / target_tps as f64);
    }
    let run_start = Instant::now();
    
    let mut stdout = io::stdout().into_raw_mode()?;
    
    let handle = thread::spawn(|| {
        let mut stdin = io::stdin();
        let mut buf = [0u8; 1];

        loop {
            if let Err(e) = stdin.read_exact(&mut buf) {
                error!("error during stdin reader loop: {}", e);
                break;
            }

            if *buf.get(0).unwrap() as char == 'q' {
                break;
            }
        }
    });
    
    print!("{}{}", ansi::HIDE_CURSOR, ansi::ENABLE_ALT_SCREEN); 
    stdout.flush()?;
    
    while !handle.is_finished() {
        let tick_start = Instant::now();
        ticks += 1;

        print!("{}{}", ansi::ERASE_SCREEN, ansi::HOME_CURSOR);

        let tps = ticks as f64 / run_start.elapsed().as_secs_f64();
        print!("tps {:.1}\n\r", tps);

        render_world(world, false)?;
        physics::tick(world)?;

        stdout.flush()?;
        let delta_time = target_mspt - tick_start.elapsed();
        if delta_time > Duration::ZERO && target_mspt != Duration::MAX {
            thread::sleep(delta_time);
        }
    }

    print!("{}{}", ansi::SHOW_CURSOR, ansi::DISABLE_ALT_SCREEN);
    stdout.flush()?;

    Ok(())
}
