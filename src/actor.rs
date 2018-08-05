
use std::sync::{Arc,Mutex};

use fp_rust::{
    common::{RawFunc,},
    sync::{CountDownLatch},
    handler::{Handler,HandlerThread},
};
use rlua::{Lua,Error,Error::RuntimeError,Function};
use message::LuaMessage;

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
        let actor: Actor = Default::default();
        actor.start();
        actor
    }
    pub fn new_with_handler(handler: Option<Arc<Mutex<HandlerThread>>>) -> Actor {
        let mut actor: Actor = Default::default();
        actor.handler = handler;
        actor.start();
        actor
    }

    pub fn lua(&self) -> Arc<Mutex<Lua>> {
        self.lua.clone()
    }
    fn start(&self) {
        match self.handler {
            Some(ref _h) => _h.lock().unwrap().start(),
            None => {},
        }
    }

    pub fn set_global(&self, key: String, value: LuaMessage) -> Result<(), Error> {
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
    fn _set_global(lua: Arc<Mutex<Lua>>, key: String, value: LuaMessage) -> Result<(), Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.set(key, value)?)
    }

    pub fn get_global(&self, key: String) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let _result : Arc<Mutex<Result<LuaMessage, Error>>> = Arc::new(Mutex::new(Err(RuntimeError(String::from("")))));
                let result : Arc<Mutex<Result<LuaMessage, Error>>> = _result.clone();
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
    fn _get_global(lua: Arc<Mutex<Lua>>, key: String) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.get::<_, LuaMessage>(key)?)
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
    fn _load(lua: Arc<Mutex<Lua>>, source: &str, name: Option<&str>) -> Result<(), Error> {
        let vm = lua.lock().unwrap();
        vm.load(source, name)?;
        Ok(())
    }
    // pub fn _load<'lua>(vm: &'lua Lua, source: &str, name: Option<&str>) -> Result<Function<'lua>, Error> {
    //     Ok(vm.load(source, name)?)
    // }
    pub fn exec(&self, source: &'static str, name: Option<&'static str>) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let _result : Arc<Mutex<Result<LuaMessage, Error>>> = Arc::new(Mutex::new(Err(RuntimeError(String::from("")))));
                let result : Arc<Mutex<Result<LuaMessage, Error>>> = _result.clone();
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
    fn _exec(lua: Arc<Mutex<Lua>>, source: &str, name: Option<&str>) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        Ok(vm.exec(source, name)?)
    }
    pub fn eval(&self, source: &'static str, name: Option<&'static str>) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let _result : Arc<Mutex<Result<LuaMessage, Error>>> = Arc::new(Mutex::new(Err(RuntimeError(String::from("")))));
                let result : Arc<Mutex<Result<LuaMessage, Error>>> = _result.clone();
                let lua = self.lua.clone();

                let done_latch = CountDownLatch::new(1);

                let done_latch2 = done_latch.clone();
                _handler.lock().unwrap().post(RawFunc::new(move ||{
                    let lua = lua.clone();
                    {
                        (*result.lock().unwrap()) = Self::_eval(lua, source, name);
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
                Self::_eval(self.lua.clone(), source, name)
            }
        }
    }
    fn _eval(lua: Arc<Mutex<Lua>>, source: &str, name: Option<&str>) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        Ok(vm.eval(source, name)?)
    }

    pub fn call(&self, name: String, args: LuaMessage) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let _result : Arc<Mutex<Result<LuaMessage, Error>>> = Arc::new(Mutex::new(Err(RuntimeError(String::from("")))));
                let result : Arc<Mutex<Result<LuaMessage, Error>>> = _result.clone();
                let lua = self.lua.clone();

                let done_latch = CountDownLatch::new(1);

                let done_latch2 = done_latch.clone();
                _handler.lock().unwrap().post(RawFunc::new(move ||{
                    let lua = lua.clone();
                    {
                        (*result.lock().unwrap()) = Self::_call(lua, name.clone(), args.clone());
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
                Self::_call(self.lua.clone(), name, args)
            }
        }
    }
    pub fn _call(lua: Arc<Mutex<Lua>>, name: String, args: LuaMessage) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        let func: Function = vm.globals().get(name)?;
        Ok(func.call::<_, LuaMessage>(args)?)
    }
}

#[test]
fn test_actor_new() {

    fn test_actor(act: Actor) -> Result<(), Error> {
        let _ = act.exec(r#"
            i = 1
        "#, None);
        assert_eq!(Some(1), Option::from(act.get_global("i".to_string())?));

        let v = act.eval(r#"
            3
        "#, None);
        assert_eq!(Some(3), Option::from(v?));

        Ok(())
    }

    let _ = test_actor(Actor::new_with_handler(None));
    let _ = test_actor(Actor::new());
}
