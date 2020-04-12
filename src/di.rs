use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type TypeMap<V> = HashMap<TypeId, V>;

type DIObj<T> = Arc<Mutex<T>>;

trait GetDep: 'static + Sized {
  fn get_dep(manager: &DIManager) -> Option<Self>;
}

trait DIBuilder {
    type Deps: GetDep;
    type Ret: 'static;

    fn build(deps: Self::Deps) -> Self::Ret;
}

struct DIManager {
  pub(crate) objs: TypeMap<Box<dyn Any>>
}

impl DIManager {
  pub fn new() -> DIManager {
    DIManager { objs: HashMap::new() }
  }

  pub fn build<T: DIBuilder>(&mut self) -> Option<DIObj<T::Ret>> {
    let deps = <T::Deps as GetDep>::get_dep(self)?;
    let obj = <T as DIBuilder>::build(deps);
    let sync_obj = Arc::new(Mutex::new(obj));
    self.objs.insert(TypeId::of::<DIObj<T::Ret>>(), Box::new(sync_obj.clone()));
    Some(sync_obj)
  }
}


impl<T: 'static> GetDep for DIObj<T> {
  fn get_dep(manager: &DIManager) -> Option<Self> {
    manager.objs.get(&TypeId::of::<Self>()).map(|obj| {
      obj.downcast_ref::<Self>().unwrap().clone()
    })
  }
}

impl GetDep for () {
  fn get_dep(manager: &DIManager) -> Option<Self> {
    Some(())
  }
}

impl<T: GetDep> GetDep for (T,) {
  fn get_dep(manager: &DIManager) -> Option<Self> {
    <T as GetDep>::get_dep(manager).map(|t| (t,))
  }
}

impl<S: GetDep, T: GetDep> GetDep for (S, T) {
  fn get_dep(manager: &DIManager) -> Option<Self> {
    <S as GetDep>::get_dep(manager).and_then(|s| {
      <T as GetDep>::get_dep(manager).and_then(|t| {
        Some((s, t))
      })
    })
  }
}

trait A {
    fn run(&self) -> i32;
}

struct A1;
struct A2;
impl A for A1 {
    fn run(&self) -> i32 {
        1
    }
}
impl A for A2 {
    fn run(&self) -> i32 {
        2
    }
}

impl DIBuilder for A1 {
  type Deps = ();
  type Ret = Box<dyn A>;
  fn build((): ()) -> Box<dyn A> {
    Box::new(A1)
  }
}

impl DIBuilder for A2 {
  type Deps = ();
  type Ret = Box<dyn A>;
  fn build((): ()) -> Box<dyn A> {
    Box::new(A2)
  }
}

struct B {
  a: DIObj<Box<dyn A>>
}

impl DIBuilder for B {
  type Deps = (DIObj<Box<dyn A>>,);
  type Ret = B;

  fn build((a,): Self::Deps) -> B {
    B { a }
  }
}

impl B {
  fn run(&self) {
    println!("A: {}", self.a.lock().unwrap().run());
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn basic() {
    let mut manager = DIManager::new();
    manager.build::<A2>().unwrap();
    let b = manager.build::<B>().unwrap();
    b.lock().unwrap().run();
  }
}
