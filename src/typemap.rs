use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct TypeMap(HashMap<TypeId, Box<dyn Any>>);

impl TypeMap {
  pub fn new() -> TypeMap {
    TypeMap(HashMap::new())
  }

  pub fn has<T: 'static + Any>(&self) -> bool {
    self.0.contains_key(&TypeId::of::<T>())
  }

  pub fn get<T: 'static + Any>(&self) -> Option<&T> {
    self
      .0
      .get(&TypeId::of::<T>())
      .map(|t| t.downcast_ref::<T>().unwrap())
  }

  pub fn get_mut<T: 'static + Any>(&mut self) -> Option<&mut T> {
    self
      .0
      .get_mut(&TypeId::of::<T>())
      .map(|t| t.downcast_mut::<T>().unwrap())
  }

  pub fn set<T: 'static + Any>(&mut self, t: T) {
    self.0.insert(TypeId::of::<T>(), Box::new(t));
  }
}
