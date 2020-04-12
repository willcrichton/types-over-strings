use crate::typemap::TypeMap;

pub trait Event: 'static {}
pub trait EventListener<E: Event> = FnMut(&E) -> () + 'static;

type ListenerVec<E> = Vec<Box<dyn EventListener<E>>>;

pub struct EventDispatcher(TypeMap);

impl EventDispatcher {
  pub fn new() -> EventDispatcher {
    EventDispatcher(TypeMap::new())
  }

  pub fn add_event_listener<E, F>(&mut self, f: F)
  where
    E: Event,
    F: EventListener<E>,
  {
    if !self.0.has::<ListenerVec<E>>() {
      self.0.set::<ListenerVec<E>>(Vec::new());
    }

    let listeners = self.0.get_mut::<ListenerVec<E>>().unwrap();
    listeners.push(Box::new(f));
  }

  pub fn trigger<E: Event>(&mut self, event: &E) {
    if let Some(listeners) = self.0.get_mut::<ListenerVec<E>>() {
      for callback in listeners {
        callback(event);
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  struct OnClick {
    mouse_x: i32,
    mouse_y: i32,
  }

  impl Event for OnClick {}

  #[test]
  fn basic() {
    let mut node = EventDispatcher::new();
    node.add_event_listener(|event: &OnClick| {
      assert_eq!(event.mouse_x, 10);
      assert_eq!(event.mouse_y, 5);
    });
    node.trigger(&OnClick {
      mouse_x: 10,
      mouse_y: 5,
    })
  }
}
