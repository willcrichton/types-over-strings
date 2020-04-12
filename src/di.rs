use crate::typemap::TypeMap;
use std::sync::{Arc, Mutex};

pub type DIObj<T> = Arc<Mutex<T>>;

pub trait GetInput: 'static + Sized {
  fn get_input(manager: &DIManager) -> Option<Self>;
}

pub trait DIBuilder {
  type Input: GetInput;
  type Output: 'static;

  fn build(input: Self::Input) -> Self::Output;
}

pub struct DIManager {
  pub(crate) objs: TypeMap,
}

impl DIManager {
  pub fn new() -> DIManager {
    DIManager {
      objs: TypeMap::new(),
    }
  }

  pub fn build<T: DIBuilder>(&mut self) -> Option<DIObj<T::Output>> {
    let deps = <T::Input as GetInput>::get_input(self)?;
    let obj = <T as DIBuilder>::build(deps);
    let sync_obj = Arc::new(Mutex::new(obj));
    self.objs.set::<DIObj<T::Output>>(sync_obj.clone());
    Some(sync_obj)
  }
}

impl<T: 'static> GetInput for DIObj<T> {
  fn get_input(manager: &DIManager) -> Option<Self> {
    manager.objs.get::<Self>().map(|obj| obj.clone())
  }
}

impl GetInput for () {
  fn get_input(_manager: &DIManager) -> Option<Self> {
    Some(())
  }
}

impl<T: GetInput> GetInput for (T,) {
  fn get_input(manager: &DIManager) -> Option<Self> {
    <T as GetInput>::get_input(manager).map(|t| (t,))
  }
}

impl<S: GetInput, T: GetInput> GetInput for (S, T) {
  fn get_input(manager: &DIManager) -> Option<Self> {
    <S as GetInput>::get_input(manager)
      .and_then(|s| <T as GetInput>::get_input(manager).and_then(|t| Some((s, t))))
  }
}

#[cfg(test)]
mod test {
  use super::*;

  trait Database {
    fn name(&self) -> &'static str;
  }

  struct MySQL;
  struct Postgres;
  impl Database for MySQL {
    fn name(&self) -> &'static str {
      "MySQL"
    }
  }
  impl Database for Postgres {
    fn name(&self) -> &'static str {
      "Postgres"
    }
  }

  impl DIBuilder for MySQL {
    type Input = ();
    type Output = Box<dyn Database>;
    fn build((): ()) -> Box<dyn Database> {
      Box::new(MySQL)
    }
  }

  impl DIBuilder for Postgres {
    type Input = ();
    type Output = Box<dyn Database>;
    fn build((): ()) -> Box<dyn Database> {
      Box::new(Postgres)
    }
  }

  struct WebServer {
    db: DIObj<Box<dyn Database>>,
  }

  impl DIBuilder for WebServer {
    type Input = (DIObj<Box<dyn Database>>,);
    type Output = WebServer;

    fn build((db,): Self::Input) -> WebServer {
      WebServer { db }
    }
  }

  impl WebServer {
    fn run(&self) {
      println!("Db name: {}", self.db.lock().unwrap().name());
    }
  }

  #[test]
  fn basic() {
    let mut manager = DIManager::new();
    manager.build::<MySQL>().unwrap();
    let server = manager.build::<WebServer>().unwrap();
    server.lock().unwrap().run();
  }
}
