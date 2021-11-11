use crate::menu::{Component};
use crate::Context;
use bsp::hal::trng::Trng;

use crate::init::debug;

use neotrellis::{Color, MultiEvent};

#[derive(Copy, Clone, Debug)]
enum Cell {
  Dead,
  Alive
}

pub struct GoL {
  generation: [[Cell;8]; 8],
  new_generation: [[Cell;8]; 8],
  stop_watch: u32,
}

fn random_cell(rng: &Trng) -> Cell {
  let random = rng.random_u8();

  let cell = match random {
    x if x > (255 / 2) as u8=> Cell::Dead,
    _ => Cell::Alive,
  };
  cell
}

impl GoL {
  pub fn new(context: &Context) -> Self {
    let mut generation = [[Cell::Dead;8]; 8];

    for x in 0..8 {
      for y in 0..8 {
        generation[x][y] = random_cell(&context.rng);
      }
    }

    // generation[0][3] = Cell::Alive;
    // generation[0][4] = Cell::Alive;
    // generation[0][5] = Cell::Alive;
    // generation[1][3] = Cell::Alive;
    // generation[1][4] = Cell::Alive;
    // generation[1][5] = Cell::Alive;

    Self {
      generation,
      new_generation: generation,
      stop_watch: context.timer
    }
  }

  fn live_neighbour_count(&self, row: u8, column: u8) -> u8 {
    let mut count = 0;
    for delta_row in [7, 0, 1].iter().cloned() {
      for delta_col in [7, 0, 1].iter().cloned() {
        if delta_row == 0 && delta_col == 0 {
          continue;
        }

        let neighbor_row = (row + delta_row) % 8;
        let neighbor_col = (column + delta_col) % 8;
        match &self.generation[neighbor_row as usize][neighbor_col as usize] {
          Cell::Dead => (),
          Cell::Alive => count += 1,
        }

      }
    }
    
    count
  }
}

impl Component for GoL {
  fn update(&mut self, _event: Option<MultiEvent>, context: &Context ) {
    if context.timer - self.stop_watch < 10 {
      return ()
    }

    debug(format!("\r\ngen {:?}", self.generation));


    for x in 0..8 {
      for y in 0..8 {
        let old_cell = self.generation[x][y];
        let live_neighbours = self.live_neighbour_count(x as u8, y as u8);
        let next_cell = match (old_cell, live_neighbours) {
          (Cell::Alive, x) if x < 2 => Cell::Dead,
          (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
          (Cell::Alive, x) if x > 3 => Cell::Dead,
          (Cell::Dead, 3) => Cell::Alive,
          (otherwise, _) => otherwise,
        };

        // debug(format!("x, y: {:?}  next: {:?}, neighbor: {:?}", (x,y), next_cell, live_neighbours));

        self.new_generation[x][y] = next_cell
      }
    }

    debug(format!("{:?}", self.new_generation));

    self.generation = self.new_generation;
    self.stop_watch = context.timer;

    // debug(format!("{:?}, watch: {:?}, timer: {:?}", self.generation, self.stop_watch, context.timer));
  }

  fn render(&self) ->  [[Option<Color>; 8]; 8] {
    let mut colors = [[Some(Color::rgb(0,0,0));8];8];

    for x in 0..8 {
      for y in 0..8 {
        match self.generation[x][y] {
          Cell::Dead => colors[x][y] = None,
          Cell::Alive => colors[x][y] = Some(Color::rgb(12,12,12))
        }
      }
    }
    colors
  }
}