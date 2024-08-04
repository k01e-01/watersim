use std::{fmt::Display, fs::File, io::{Read, Write}, path::Path};

use anyhow::Result;
use bincode::deserialize;
use log::error;
use serde::{Deserialize, Serialize};

use crate::ansi;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum TileKind {
    #[default]
    Empty,
    Water(f32),
    Wall,
}

impl Display for TileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}  {}", match &self {
            TileKind::Empty                 => ansi::RESET.to_string(),
            TileKind::Wall                  => ansi::WHITE_BG.to_string(),

            TileKind::Water(..1.0)          => ansi::blue(16*7),
            TileKind::Water(1.0..2.0)       => ansi::blue(16*6),
            TileKind::Water(2.0..3.0)       => ansi::blue(16*5),
            TileKind::Water(3.0..4.0)       => ansi::blue(16*4),
            TileKind::Water(4.0..5.0)       => ansi::blue(16*3),
            TileKind::Water(5.0..6.0)       => ansi::blue(16*2),
            TileKind::Water(6.0..7.0)       => ansi::blue(16*1),
            TileKind::Water(7.0..)          => ansi::blue(16*0),

            TileKind::Water(_)              => ansi::RED_BG.to_string(),
        }, ansi::RESET))
    }
}

pub type Grid = Box<[Box<[TileKind]>]>;

fn init_filled_array2d<T>(width: u16, height: u16) -> Box<[Box<[T]>]> 
    where T: Default + Clone 
{
    let mut outer_vec: Vec<Box<[T]>> = Vec::with_capacity(height.into());

    for _ in 0..height {
        let inner_vec: Vec<T> = vec![T::default(); width.into()];
        outer_vec.push(inner_vec.into_boxed_slice());
    }

    outer_vec.into_boxed_slice()
}

pub fn new_grid(width: u16, height: u16) -> Grid {
    init_filled_array2d(width, height)
}

pub fn save_grid(file: &Path, grid: &Grid) -> Result<()> {
    let mut file = match File::create(file) {
        Ok(f) => f,
        Err(e) => {
            error!("failed to open savefile: {}", e);
            return Ok(())
        }
    };

    file.write_all(&bincode::serialize(grid)?)?;
    Ok(())
}

pub fn load_grid(file: &Path, grid: &mut Grid) -> Result<()> {
    let mut file = match File::open(file) {
        Ok(f) => f,
        Err(e) => {
            error!("failed to open savefile: {}", e);
            return Ok(())
        }
    };

    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    *grid = deserialize(&data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::{init_filled_array2d, TileKind};

    #[test]
    fn init_filled_array2d_works() {
        let out: Box<[Box<[()]>]> = init_filled_array2d(32, 32);

        assert_eq!(out.len(), 32);

        for row in out.iter() {
            assert_eq!(row.len(), 32);

            for val in row.iter() {
                assert_eq!(val, &());
            }
        }
    }

    #[bench]
    fn init_filled_array2d_bench(b: &mut test::Bencher) {
        b.iter(|| {
           test::black_box(init_filled_array2d::<u32>(255, 255)); 
        });
    }
}
