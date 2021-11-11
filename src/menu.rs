use alloc::rc::Rc;
use alloc::vec;

use crate::init::debug;
use crate::Context;
use core::cell::RefCell;
use core::convert::TryInto;

use crate::gol::GoL;

use neotrellis::{Color, Event, MultiEvent};

pub trait Component {
  fn update(&mut self, event: Option<MultiEvent>, context: &Context);
  fn render(&self) -> [[Option<Color>; 8]; 8];
}

#[derive(Debug)]
struct Exit {
  exit_presses: [u32; 3],
  count: u8,
}

impl Default for Exit {
  fn default() -> Exit {
    Exit {
      exit_presses: [u32::MAX; 3],
      count: 0,
    }
  }
}

pub struct MenuComponent {
  exit: Exit,
  current_component: Option<Rc<RefCell<dyn Component>>>,
  components: vec::Vec<Rc<RefCell<dyn Component>>>,
}

impl MenuComponent {
  pub fn new(context: &Context) -> Self {
    let paint = Rc::new(RefCell::new(PaintComponent { steps: [[0; 8]; 8] }));
    let gol = Rc::new(RefCell::new(GoL::new(context)));

    Self {
      current_component: None,
      exit: Exit::default(),
      components: vec![paint, gol],
    }
  }
}

impl Component for MenuComponent {
  fn update(&mut self, event: Option<MultiEvent>, context: &Context) {
    if let Some(event) = event {
      match (self.current_component.as_ref(), event) {
        (_, MultiEvent {
          coordinate: (0, 0),
          event: Event::Rising,
        }) => {
          self.exit.count += 1;
          self.exit.exit_presses[2] = self.exit.exit_presses[1];
          self.exit.exit_presses[1] = self.exit.exit_presses[0];
          self.exit.exit_presses[0] = context.timer;
        }
        (None, MultiEvent {
          coordinate: (1, 2),
          event: Event::Rising,
        }) => self.current_component = Some(Rc::clone(&self.components[0])),
        (None, MultiEvent {
          coordinate: (1, 3),
          event: Event::Rising,
        }) => {
          self.components[1] = Rc::new(RefCell::new(GoL::new(context)));
          self.current_component = Some(Rc::clone(&self.components[1]))
        }
        _ => (),
      }
    }

    // debug(format! {"counter: {:?}, {:?} timer:{:?}", self.exit, self.exit_presses, timer});
    if self.exit.count > 2 {
      if context.timer - self.exit.exit_presses[2] < 20 {
        self.current_component = None;
      }
      self.exit = Exit::default(); //ToDo: Sometimes 5 clicks needed to escape
    }

    match &self.current_component {
      Some(component) => component.borrow_mut().update(event, context),
      None => (),
    }
  }

  
  fn render(&self) -> [[Option<Color>; 8]; 8] {
    let mut menu_render = [[Some(Color::rgb(0, 0, 0)); 8]; 8];

    menu_render[1][2] = Some(Color::rgb(0,0,255));
    menu_render[1][3] = Some(Color::rgb(0,255,0));

    match &self.current_component {
      Some(component) => component.borrow().render(),
      None => menu_render,
    }
  }
}

struct PaintComponent {
  steps: [[u8; 8]; 8],
}

impl Component for PaintComponent {
  fn update(&mut self, event: Option<MultiEvent>, context: &Context) {
    if let Some(MultiEvent {
      coordinate: (x, y),
      event: Event::Rising,
    }) = event
    {
      let xi: usize = x.try_into().unwrap();
      let yi: usize = y.try_into().unwrap();
      let step = &mut self.steps[xi][yi];
      if *step == 3 {
        *step = 0;
      } else {
        *step += 1;
      }
    }
  }

  fn render(&self) -> [[Option<Color>; 8]; 8] {
    let mut colors: [[Option<Color>; 8]; 8] = [[None; 8]; 8];
    for (x, item) in self.steps.iter().enumerate() {
      for (y, step) in item.iter().enumerate() {
        match step {
          1 => colors[x][y] = Some(Color::rgb(125, 0, 0)),
          2 => colors[x][y] = Some(Color::rgb(0, 125, 0)),
          3 => colors[x][y] = Some(Color::rgb(0, 0, 125)),
          _ => (),
        }
      }
    }

    colors
  }
}
