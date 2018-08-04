extern crate rlua;

use std::sync::{Arc,Mutex};

use actor::rlua::{Lua,UserData,Error};

#[derive(Clone)]
pub struct Actor {
    is_async: bool,
    lua: Arc<Mutex<Lua>>,
}

impl Default for Actor {
    fn default() -> Self {
        Actor{
            is_async: true,
            lua: Arc::new(Mutex::new(Lua::new())),
        }
    }
}

impl Actor {
    pub fn new() -> Actor {
        Default::default()
    }

    pub fn new_with_param(is_async: bool) -> Actor {
        let mut actor: Actor = Default::default();
        actor.is_async = is_async;
        actor
    }

    pub fn lua(&self) -> Arc<Mutex<Lua>> {
        self.lua.clone()
    }

    pub fn set_global<T: UserData + Send>(&self, key: String, value: T) -> Result<(), Error> {
        self._set_global(key, value)
    }
    fn _set_global<T: UserData + Send>(&self, key: String, value: T) -> Result<(), Error> {
        let vm = self.lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.set(key, value)?)
    }

    pub fn get_global<T: UserData + Clone>(&self, key: String) -> Result<T, Error> {
        self._get_global(key)
    }
    fn _get_global<T: UserData + Clone>(&self, key: String) -> Result<T, Error> {
        let vm = self.lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.get::<_, T>(key)?)
    }

    // pub fn load(&self, source: &str, name: Option<&str>) -> Result<Function, Error> {
    //     self._load(source, name)
    // }
    // pub fn _load(&self, source: &str, name: Option<&str>) -> Result<Function, Error> {
    //     let vm = self.lua.lock().unwrap();
    //     Ok(vm.load(source, name)?)
    // }
    pub fn exec<T: UserData + Clone>(&self, source: &str, name: Option<&str>) -> Result<T, Error> {
        self._exec(source, name)
    }
    pub fn _exec<T: UserData + Clone>(&self, source: &str, name: Option<&str>) -> Result<T, Error> {
        let vm = self.lua.lock().unwrap();
        Ok(vm.exec(source, name)?)
    }
}
