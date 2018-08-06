
use std::sync::{Arc,Mutex};

use fp_rust::{
    common::{RawFunc,},
    sync::{CountDownLatch},
    handler::{Handler,HandlerThread},
};
use rlua::{Lua,Error,Error::RuntimeError,Function,FromLuaMulti,ToLuaMulti};
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
    fn wait_async_lua_message_result(&self, _handler: Arc<Mutex<HandlerThread>>, func: impl FnOnce()->Result<LuaMessage, Error> + Send + Sync + 'static + Clone) -> Result<LuaMessage, Error> {
        let func = Arc::new(Mutex::new(func));

        let _result : Arc<Mutex<Result<LuaMessage, Error>>> = Arc::new(Mutex::new(Err(RuntimeError(String::from("")))));
        let result : Arc<Mutex<Result<LuaMessage, Error>>> = _result.clone();

        let done_latch = CountDownLatch::new(1);

        let done_latch2 = done_latch.clone();
        _handler.lock().unwrap().post(RawFunc::new(move ||{
            {
                (*result.lock().unwrap()) = (func.lock().unwrap().clone())();
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
    }

    pub fn set_global(&self, key: &'static str, value: LuaMessage) -> Result<(), Error> {
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
    fn _set_global(lua: Arc<Mutex<Lua>>, key: &str, value: LuaMessage) -> Result<(), Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.set(key, value)?)
    }

    pub fn get_global(&self, key: &'static str) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(_handler, move ||{
                    Self::_get_global(lua, key.clone())
                })
            },
            None => {
                Self::_get_global(self.lua.clone(), key)
            },
        }
    }
    fn _get_global(lua: Arc<Mutex<Lua>>, key: &str) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.get::<_, LuaMessage>(key)?)
    }
    pub fn def_fn<'lua, 'callback, F, A, R>(lua: &'lua Lua, func: F) -> Result<Function<'lua>, Error>
        where
            A: FromLuaMulti<'callback>,
            R: ToLuaMulti<'callback>,
            F: 'static + Send + Fn(&'callback Lua, A) -> Result<R, Error>
    {
        Ok(lua.create_function(func)?)
    }
    pub fn def_fn_with_name<'lua, 'callback, F, A, R>(lua: &'lua Lua, func: F, key: &str) -> Result<Function<'lua>, Error>
        where
            A: FromLuaMulti<'callback>,
            R: ToLuaMulti<'callback>,
            F: 'static + Send + Fn(&'callback Lua, A) -> Result<R, Error>
    {
        let def = lua.create_function(func)?;
        lua.globals().set(key, def)?;
        Ok(lua.globals().get::<_, Function<'lua>>(key)?)
    }

    pub fn load<'lua>(lua: &'lua Lua, source: &str, name: Option<&str>) -> Result<Function<'lua>, Error> {
        let vm = lua;
        Ok(vm.load(source, name)?)
    }
    // pub fn _load<'lua>(vm: &'lua Lua, source: &str, name: Option<&str>) -> Result<Function<'lua>, Error> {
    //     Ok(vm.load(source, name)?)
    // }
    pub fn exec(&self, source: &'static str, name: Option<&'static str>) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(_handler, move ||{
                    Self::_exec(lua, source.clone(), name.clone())
                })
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
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(_handler, move ||{
                    Self::_eval(lua, source.clone(), name.clone())
                })
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

    pub fn call(&self, name: &'static str, args: LuaMessage) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(_handler, move ||{
                    Self::_call(lua, name.clone(), args.clone())
                })
            },
            None => {
                Self::_call(self.lua.clone(), name, args)
            }
        }
    }
    pub fn _call(lua: Arc<Mutex<Lua>>, name: &str, args: LuaMessage) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        let func: Function = vm.globals().get::<_, Function>(name)?;

        Ok(func.call::<_, LuaMessage>(args)?)
    }
}

#[test]
fn test_actor_new() {

    use rlua::{Variadic};

    fn test_actor(act: Actor) {
        let _ = act.exec(r#"
            i = 1
        "#, None);
        assert_eq!(Some(1), Option::from(act.get_global("i").ok().unwrap()));

        let v = act.eval(r#"
            3
        "#, None);
        assert_eq!(Some(3), Option::from(v.ok().unwrap()));

        act.exec(r#"
            function testit (i)
                return i + 1
            end
        "#, None).ok().unwrap();
        match act.call("testit", LuaMessage::from(1)) {
            Ok(_v) => {
                assert_eq!(Some(2), Option::from(_v));
            },
            Err(_err) => {
                println!("{:?}", _err);
                panic!(_err);
            },
        }

        {
            let vm = act.lua();
            let vm = vm.lock().unwrap();
            Actor::def_fn_with_name(&vm, |_, (list1, list2): (Vec<String>, Vec<String>)| {
                // This function just checks whether two string lists are equal, and in an inefficient way.
                // Lua callbacks return `rlua::Result`, an Ok value is a normal return, and an Err return
                // turns into a Lua 'error'.  Again, any type that is convertible to lua may be returned.
                Ok(list1 == list2)
            }, "check_equal").ok().unwrap();
            Actor::def_fn_with_name(&vm, |_, strings: Variadic<String>| {
                // (This is quadratic!, it's just an example!)
                Ok(strings.iter().fold("".to_owned(), |a, b| a + b))
            }, "join").ok().unwrap();
            assert_eq!(
                vm.eval::<bool>(r#"check_equal({"a", "b", "c"}, {"a", "b", "c"})"#, None).ok().unwrap(),
                true
            );
            assert_eq!(
                vm.eval::<bool>(r#"check_equal({"a", "b", "c"}, {"d", "e", "f"})"#, None).ok().unwrap(),
                false
            );
            assert_eq!(vm.eval::<String>(r#"join("a", "b", "c")"#, None).ok().unwrap(), "abc");
        }

        act.set_global("arr1", LuaMessage::from(vec!(LuaMessage::from(1), LuaMessage::from(2)))).ok().unwrap();

        let v = Option::<Vec<LuaMessage>>::from(act.get_global("arr1").ok().unwrap());
        assert_eq!(LuaMessage::from(1), v.clone().unwrap()[0]);
        assert_eq!(LuaMessage::from(2), v.clone().unwrap()[1]);
        assert_eq!(2, v.clone().unwrap().len());
    }

    let _ = test_actor(Actor::new_with_handler(None));
    let _ = test_actor(Actor::new());
}
