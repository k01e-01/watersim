#![feature(test)]
#![feature(duration_millis_float)]

mod ansi;
mod grid;
mod physics;
mod ui;

use anyhow::Result;
use grid::Grid;
use log::{debug, error, info};
use rustyline::history::MemHistory;
use ui::UserAction;

const DEF_WORLD_WIDTH: u16 = 8;
const DEF_WORLD_HEIGHT: u16 = 8;


fn main() -> Result<()> {
    env_logger::builder()
        .filter_module("rustyline", log::LevelFilter::Error)
        .init();
    info!("hello world");
    
    let mut world: Grid = grid::new_grid(DEF_WORLD_WIDTH, DEF_WORLD_HEIGHT);
    let mut tps: u16 = 60;

    let mut rl = rustyline::Editor::with_history(
        rustyline::Config::default(), 
        MemHistory::new(),
    )?;

    loop {
        let action = ui::request_user_action(&mut rl)?;

        match action {
            UserAction::Noop => (),
            UserAction::Quit => break,
            UserAction::Show => ui::render_world(&world, true)?,
            UserAction::Help => ui::show_help(),

            UserAction::Inspect(x, y) => 
                info!("({}, {}) = {2} ({2:?})", x, y, world[y as usize][x as usize]),

            UserAction::New(w, h) => world = grid::new_grid(w, h),
            UserAction::Save(path) => grid::save_grid(&path, &world)?,
            UserAction::Load(path) => grid::load_grid(&path, &mut world)?,

            UserAction::Tps(n) => tps = n,
            UserAction::Set(x, y, t) => {
                if y as usize >= world.len() 
                    || x as usize >= world[y as usize].len() 
                {
                    error!("out of bounds!")
                } else {
                    world[y as usize][x as usize] = t
                }
            },

            UserAction::Tick => {
                if let Err(e) = physics::tick(&mut world) {
                    error!("fatal error during tick: {}", e);
                    break;
                }
                ui::render_world(&world, true)?;
            },
            UserAction::Run => {
                if let Err(e) = ui::run_loop(&mut world, tps) {
                    error!("fatal error during tick: {}", e);
                    break;
                }
            },
        };
    }

    debug!("goodbye!");

    Ok(())
}

