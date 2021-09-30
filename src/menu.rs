use alloc::rc::Rc;
use alloc::vec;


use core::cell::RefCell;
use core::convert::TryInto;

use neotrellis::{Color, Event, MultiEvent};

pub trait Component {
  fn update(&mut self, event: MultiEvent);
  fn render(&self) -> [[Option<Color>; 8]; 8];
}

struct Exit {
  elapsed: usize,
  count: u8,
}

pub struct MenuComponent {
  exit: Exit,
  current_component: Option<Rc<RefCell<dyn Component>>>,
  components: vec::Vec<Rc<RefCell<dyn Component>>>,
}

impl MenuComponent {
  pub fn new() -> Self {
    let paint = Rc::new(RefCell::new(PaintComponent { steps: [[0; 8]; 8] }));

    Self {
      current_component: None,
      exit: Exit {
        elapsed: 0,
        count: 0,
      },
      components: vec![paint],
    }
  }
}

impl Component for MenuComponent {
  fn update(&mut self, event: MultiEvent) {
    if let MultiEvent {
      coordinate: (_x, _y),
      event: Event::Rising,
    } = event
    {
      self.current_component = Some(Rc::clone(&self.components[0]));
    }

    match &self.current_component {
      Some(component) => component.borrow_mut().update(event),
      None => (),
    }
  }

  fn render(&self) -> [[Option<Color>; 8]; 8] {
    match &self.current_component {
      Some(component) => component.borrow().render(),
      None => [[Some(Color::rgb(0, 0, 10)); 8]; 8],
    }
  }
}

struct PaintComponent {
  steps: [[u8; 8]; 8],
}

impl Component for PaintComponent {
  fn update(&mut self, event: MultiEvent) {
    if let MultiEvent {
      coordinate: (x, y),
      event: Event::Rising,
    } = event
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
