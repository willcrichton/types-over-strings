use std::any::Any;
use crate::typemap::TypeMap;

trait Event: 'static {}

trait EventListener<E: Event>: Any {
  fn on_event(&mut self, params: &E);
}

type ListenerVec<E> = Vec<Box<dyn EventListener<E>>>;

struct DOMNode {
  listeners: TypeMap,
}

impl DOMNode {
  pub fn new() -> DOMNode {
    DOMNode {
      listeners: TypeMap::new(),
    }
  }

  fn add_event_listener<E, F>(&mut self, f: F)
  where
    E: Event,
    F: EventListener<E>,
  {
    if !self.listeners.has::<ListenerVec<E>>() {
      self.listeners.set::<ListenerVec<E>>(Vec::new());
    }

    let listeners = self.listeners.get_mut::<ListenerVec<E>>().unwrap();
    listeners.push(Box::new(f));
  }

  fn trigger<E: Event>(&mut self, event: &E) {
    if let Some(listeners) = self.listeners.get_mut::<ListenerVec<E>>() {
      for l in listeners {
        l.on_event(event);
      }
    }
  }
}

impl<E, T> EventListener<E> for T
where
  E: Event,
  T: FnMut(&E) -> () + 'static,
{
  fn on_event(&mut self, params: &E) {
    self(params);
  }
}

struct OnClick {
  mouse_x: f32,
  mouse_y: f32,
}

impl Event for OnClick {}

fn main() {
  let mut node = DOMNode::new();
  node.add_event_listener(|event: &OnClick| {
    println!("{}, {}", event.mouse_x, event.mouse_y);
  });
  node.trigger(&OnClick {
    mouse_x: 1.0,
    mouse_y: 0.5,
  })
}
