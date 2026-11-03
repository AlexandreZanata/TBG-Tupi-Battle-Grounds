//! Uniform spatial hash rebuilt every tick so neighbour queries stay O(local).

use crate::geom::{Pos, MM};
use crate::map::SIZE;

const CELL: i32 = 8 * MM;
const DIM: i32 = (SIZE * MM + CELL - 1) / CELL;

#[derive(Clone, Debug, Default)]
pub struct Grid {
    cells: Vec<Vec<u32>>,
}

impl Grid {
    fn cell(p: i32) -> i32 {
        (p / CELL).clamp(0, DIM - 1)
    }

    /// Re-buckets `positions` (index = unit slot) keeping allocations.
    pub fn rebuild(&mut self, positions: impl Iterator<Item = Pos>) {
        if self.cells.is_empty() {
            self.cells = vec![Vec::new(); (DIM * DIM) as usize];
        }
        self.cells.iter_mut().for_each(Vec::clear);
        for (i, p) in positions.enumerate() {
            let c = Self::cell(p.y) * DIM + Self::cell(p.x);
            self.cells[c as usize].push(i as u32);
        }
    }

    /// Calls `f` with every slot whose cell overlaps the square around `p`;
    /// candidates come out in ascending cell order, slot order inside a cell.
    pub fn near(&self, p: Pos, radius: i32, mut f: impl FnMut(usize)) {
        let (x0, x1) = (Self::cell(p.x - radius), Self::cell(p.x + radius));
        let (y0, y1) = (Self::cell(p.y - radius), Self::cell(p.y + radius));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                for &i in &self.cells[(cy * DIM + cx) as usize] {
                    f(i as usize);
                }
            }
        }
    }
}
