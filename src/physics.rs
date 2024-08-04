use anyhow::{bail, Result};
use log::debug;

use crate::grid::{new_grid, Grid, TileKind};

fn add_to(next_grid: &mut Grid, x: usize, y: usize, p: f32) -> Result<()> {
    match next_grid[y][x] {
        TileKind::Wall => bail!("tried to add pressure to wall!"),
        TileKind::Empty => next_grid[y][x] = match p {
            ..=0.0 => TileKind::Empty,
            _ => TileKind::Water(p)
        },
        TileKind::Water(curr_p) => next_grid[y][x] = match p + curr_p {
            ..=0.0 => TileKind::Empty,
            _ => TileKind::Water(p + curr_p),
        },
    }

    Ok(()) 
}

fn get_pressure(tile: TileKind) -> f32 {
    match tile {
        TileKind::Wall => f32::INFINITY,
        TileKind::Empty => 0.0,
        TileKind::Water(p) => p,
    }
}

fn neighbours(grid: &Grid, x: usize, y: usize) -> (f32, f32, f32, f32) {
    // i love dodging oob errors 
    // all tiles out of bounds are walls

    let u_p = if y == 0 {
        get_pressure(TileKind::Wall)
    } else {
        get_pressure(grid[y-1][x])
    };

    let l_p = if x == 0 {
        get_pressure(TileKind::Wall)
    } else {
        get_pressure(grid[y][x-1])
    };

    let down = grid.get(y+1).map(|l| l[x]);
    let d_p = if let Some(down_unwrapped) = down {
        get_pressure(down_unwrapped)
    } else {
        get_pressure(TileKind::Wall)
    };

    let right = grid[y].get(x+1);
    let r_p = if let Some(right_unwrapped) = right {
        get_pressure(*right_unwrapped)
    } else {
        get_pressure(TileKind::Wall)
    };

    (u_p, l_p, r_p, d_p)
}

fn flow(
    grid: &Grid, 
    next_grid: &mut Grid, 
    x: usize, 
    y: usize, 
    curr_p: f32,
) -> Result<()> {

    // u = up, l = left, r = right, d = down
    //
    //  ... u_p ...
    //  l_p @@@ r_p
    //  ... d_p ...

    // pressures
    let (u_p, l_p, r_p, d_p) = neighbours(grid, x, y);

    let mut next_p = curr_p;

    'distrib: {
        if next_p > d_p - 1.0 {
            let flow = 0.0f32.max(next_p - d_p);
            debug!("flow_d {}", flow);

            add_to(next_grid, x, y+1, flow)?;
            next_p -= flow;

            if next_p <= 0.0 {
                break 'distrib;
            }
        }

        if next_p > l_p {
            let flow = 0.0f32.max((next_p - l_p) / 3.0);
            debug!("flow_l {}", flow);

            add_to(next_grid, x-1, y, flow)?;
            next_p -= flow;

            if next_p <= 0.0 {
                break 'distrib;
            }
        }

        if next_p > r_p {
            let flow = 0.0f32.max((next_p - r_p) / 3.0);
            debug!("flow_r {}", flow);

            add_to(next_grid, x+1, y, flow)?;
            next_p -= flow;

            if next_p <= 0.0 {
                break 'distrib;
            }
        }


        if next_p > u_p + 1.0 {
            let flow = 0.0f32.max(next_p - u_p - 1.0);
            debug!("flow_u {}", flow);

            add_to(next_grid, x, y-1, flow)?;
            next_p -= flow;

            if next_p <= 0.0 {
                break 'distrib;
            }
        }

        break 'distrib;
    }

    next_p = 0.0f32.max(next_p);
    add_to(next_grid, x, y, next_p)?;

    Ok(()) 
}

pub fn tick(grid: &mut Grid) -> Result<()> {
    let width = grid.get(0).unwrap().len();
    let height = grid.len();

    let mut next_grid = new_grid(width as u16, height as u16);

    for (y, row) in grid.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            match tile {
                TileKind::Wall => next_grid[y][x] = TileKind::Wall,
                TileKind::Empty => add_to(&mut next_grid, x, y, 0.0)?,
                TileKind::Water(p) => flow(grid, &mut next_grid, x, y, *p)?,
            }
        }
    }

    *grid = next_grid;

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate test;

    use crate::grid::new_grid;

    use super::*;
    
    #[bench]
    fn tick_bench(b: &mut test::Bencher) {
        let mut world = new_grid(32, 32);

        b.iter(|| {
            tick(&mut world).expect("tick failed during bench")
        })
    }
}
