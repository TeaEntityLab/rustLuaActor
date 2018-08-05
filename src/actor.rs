
use std::sync::{Arc,Mutex};

use fp_rust::{
    common::{RawFunc,},
    sync::{CountDownLatch},
    handler::{Handler,HandlerThread},
};
use rlua::{Lua,UserData,Error,Error::RuntimeError};

// pub struct TypeI8(i8);
// impl UserData for TypeI8 {}
// pub struct TypeI16(i16);
// impl UserData for TypeI16 {}
// pub struct TypeI32(i32);
// impl UserData for TypeI32 {}
// pub struct TypeI64(i64);
// impl UserData for TypeI64 {}
// pub struct TypeI128(i128);
// impl UserData for TypeI128 {}
//
// pub struct TypeF32(f32);
// impl UserData for TypeF32 {}
// pub struct TypeF64(f64);
// impl UserData for TypeF64 {}

#[derive(Clone)]
pub struct Actor {
    handler: Option<Arc<Mutex<HandlerThread>>>,
    lua: Arc<Mutex<Lua>>,
}

impl Default for Actor {
    fn default() -> Self {
        Actor{
            handler: Some(HandlerThread::new_with_mutex()),
            lua: Arc::new(Mutex::new(Lua::new())),
        }
    }
}

impl Actor {
    pub fn new() -> Actor {
        Default::default()
    }

    pub fn new_with_handler(handler: Option<Arc<Mutex<HandlerThread>>>) -> Actor {
        let mut actor: Actor = Default::default();
        actor.handler = handler;
        actor
    }

    pub fn lua(&self) -> Arc<Mutex<Lua>> {
        self.lua.clone()
    }

    pub fn set_global<T: UserData + Send + Sync + Clone>(&self, key: String, value: T) -> Result<(), Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                _handler.lock().unwrap().post(RawFunc::new(move ||{
                    let lua = lua.clone();
                    let _ = Self::_set_global(lua, key.clone(), value.clone());
                }));
                return Ok(());
            },
            None => {
                Self::_set_global(self.lua.clone(), key, value)
            }
        }
    }
    fn _set_global<T: UserData + Send>(lua: Arc<Mutex<Lua>>, key: String, value: T) -> Result<(), Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.set(key, value)?)
    }

    pub fn get_global<T: UserData + Send + Clone>(&self, key: String) -> Result<T, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let _result : Arc<Mutex<Result<T, Error>>> = Arc::new(Mutex::new(Err(RuntimeError(String::from("")))));
                let result : Arc<Mutex<Result<T, Error>>> = _result.clone();
                let lua = self.lua.clone();

                let done_latch = CountDownLatch::new(1);

                let done_latch2 = done_latch.clone();
                _handler.lock().unwrap().post(RawFunc::new(move ||{
                    let lua = lua.clone();
                    {
                        let result = result.clone();

                        {
                            (*result.lock().unwrap()) = Self::_get_global(lua, key.clone());
                        }
                        done_latch2.countdown();
                    }
                }));

                done_latch.wait();

                {
                    let _result = &*_result.lock().unwrap();
                    match _result {
                        Ok(_result) => {
                            Ok(_result.clone())
                        },
                        Err(_err) => {
                            Err(_err.clone())
                        }
                    }
                }
            },
            None => {
                Self::_get_global(self.lua.clone(), key)
            },
        }
    }
    fn _get_global<T: UserData + Clone>(lua: Arc<Mutex<Lua>>, key: String) -> Result<T, Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.get::<_, T>(key)?)
    }

    pub fn load(&self, source: &'static str, name: Option<&'static str>) -> Result<(), Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                _handler.lock().unwrap().post(RawFunc::new(move ||{
                    let lua = lua.clone();
                    let _ = Self::_load(lua, source, name);
                }));
                Ok(())
            },
            None => {
                Self::_load(self.lua.clone(), source, name)
            }
        }
    }
    pub fn _load(lua: Arc<Mutex<Lua>>, source: &str, name: Option<&str>) -> Result<(), Error> {
        let vm = lua.lock().unwrap();
        vm.load(source, name)?;
        Ok(())
    }
    // pub fn _load<'lua>(vm: &'lua Lua, source: &str, name: Option<&str>) -> Result<Function<'lua>, Error> {
    //     Ok(vm.load(source, name)?)
    // }
    pub fn exec<T: UserData + Send + Clone>(&self, source: &'static str, name: Option<&'static str>) -> Result<T, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let _result : Arc<Mutex<Result<T, Error>>> = Arc::new(Mutex::new(Err(RuntimeError(String::from("")))));
                let result : Arc<Mutex<Result<T, Error>>> = _result.clone();
                let lua = self.lua.clone();

                let done_latch = CountDownLatch::new(1);

                let done_latch2 = done_latch.clone();
                _handler.lock().unwrap().post(RawFunc::new(move ||{
                    let lua = lua.clone();
                    {
                        (*result.lock().unwrap()) = Self::_exec(lua, source, name);
                    }
                    done_latch2.countdown();
                }));

                done_latch.wait();

                {
                    let _result = &*_result.lock().unwrap();
                    match _result {
                        Ok(_result) => {
                            Ok(_result.clone())
                        },
                        Err(_err) => {
                            Err(_err.clone())
                        }
                    }
                }
            },
            None => {
                Self::_exec(self.lua.clone(), source, name)
            }
        }
    }
    pub fn _exec<T: UserData + Clone>(lua: Arc<Mutex<Lua>>, source: &str, name: Option<&str>) -> Result<T, Error> {
        let vm = lua.lock().unwrap();
        Ok(vm.exec(source, name)?)
    }
}

#[test]
fn test_handler_new() {
    use std::time;
    use std::thread;

    let act = Actor::new();

    let _ = act.exec(r#"
        var i = 3;
    "#, None);

    thread::sleep(time::Duration::from_millis(100));

    // assert_eq!(3, act.get_global::<i64>(String::from("i")).ok().unwrap());
}
