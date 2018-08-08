use std::sync::{Arc, Mutex};

use fp_rust::{
    common::{RawFunc, SubscriptionFunc},
    handler::{Handler, HandlerThread},
    sync::{CountDownLatch, Will, WillAsync},
};
use message::LuaMessage;
use rlua::{Error, FromLuaMulti, Function, Lua, ToLuaMulti};

#[derive(Clone)]
pub struct Actor {
    handler: Option<Arc<Mutex<HandlerThread>>>,
    lua: Arc<Mutex<Lua>>,
}

impl Default for Actor {
    fn default() -> Self {
        Actor {
            handler: Some(HandlerThread::new_with_mutex()),
            lua: Arc::new(Mutex::new(Lua::new())),
        }
    }
}

impl Drop for Actor {
    fn drop(&mut self) {
        self.stop_handler();
    }
}

impl Actor {
    pub fn new() -> Actor {
        let actor: Actor = Default::default();
        actor.start_handler();
        actor
    }
    pub fn new_with_handler(handler: Option<Arc<Mutex<HandlerThread>>>) -> Actor {
        let mut actor: Actor = Default::default();
        actor.handler = handler;
        actor.start_handler();
        actor
    }

    #[inline]
    pub fn lua(&self) -> Arc<Mutex<Lua>> {
        self.lua.clone()
    }
    #[inline]
    pub fn set_lua(&mut self, lua: Arc<Mutex<Lua>>) {
        self.lua = lua;
    }
    #[inline]
    fn stop_handler(&self) {
        if let Some(ref _h) = self.handler {
            _h.lock().unwrap().stop()
        }
    }
    #[inline]
    fn start_handler(&self) {
        if let Some(ref _h) = self.handler {
            _h.lock().unwrap().start()
        }
    }
    #[inline]
    fn wait_async_lua_message_result(
        &self,
        _handler: &Arc<Mutex<HandlerThread>>,
        func: impl FnOnce() -> Result<LuaMessage, Error> + Send + Sync + 'static + Clone,
    ) -> Result<LuaMessage, Error> {
        let func = Arc::new(Mutex::new(func));

        let done_latch = CountDownLatch::new(1);
        let done_latch2 = done_latch.clone();

        let mut will =
            WillAsync::new_with_handler(move || (func.lock().unwrap().clone())(), _handler.clone());
        will.add_callback(Arc::new(Mutex::new(SubscriptionFunc::new(move |_| {
            done_latch2.countdown();
        }))));
        will.start();
        done_latch.wait();

        will.result().unwrap()
    }

    pub fn set_global(&self, key: &'static str, value: LuaMessage) -> Result<(), Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                _handler.lock().unwrap().post(RawFunc::new(move || {
                    let lua = lua.clone();
                    let _ = Self::_set_global(&lua, key, value.clone());
                }));
                Ok(())
            }
            None => Self::_set_global(&self.lua.clone(), key, value),
        }
    }
    #[inline]
    fn _set_global(lua: &Arc<Mutex<Lua>>, key: &str, value: LuaMessage) -> Result<(), Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        globals.set(key, value)?;
        Ok(())
    }

    pub fn get_global(&self, key: &'static str) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(&_handler, move || Self::_get_global(&lua, key))
            }
            None => Self::_get_global(&self.lua.clone(), key),
        }
    }
    #[inline]
    fn _get_global(lua: &Arc<Mutex<Lua>>, key: &str) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        let globals = vm.globals();
        Ok(globals.get::<_, LuaMessage>(key)?)
    }
    #[inline]
    fn _get_global_function<'lua>(lua: &'lua Lua, key: &str) -> Result<Function<'lua>, Error> {
        Ok(lua.globals().get::<_, Function<'lua>>(key)?)
    }

    #[inline]
    pub fn def_fn<'lua, 'callback, F, A, R>(
        lua: &'lua Lua,
        func: F,
    ) -> Result<Function<'lua>, Error>
    where
        A: FromLuaMulti<'callback>,
        R: ToLuaMulti<'callback>,
        F: 'static + Send + Fn(&'callback Lua, A) -> Result<R, Error>,
    {
        Ok(lua.create_function(func)?)
    }
    #[inline]
    pub fn def_fn_with_name<'lua, 'callback, F, A, R>(
        lua: &'lua Lua,
        func: F,
        key: &str,
    ) -> Result<Function<'lua>, Error>
    where
        A: FromLuaMulti<'callback>,
        R: ToLuaMulti<'callback>,
        F: 'static + Send + Fn(&'callback Lua, A) -> Result<R, Error>,
    {
        let def = Self::def_fn(lua, func)?;
        lua.globals().set(key, def)?;
        Ok(Self::_get_global_function(lua, key)?)
    }
    pub fn def_fn_with_name_nowait<'callback, F, A, R>(
        &self,
        func: F,
        key: &'static str,
    ) -> Result<(), Error>
    where
        A: FromLuaMulti<'callback>,
        R: ToLuaMulti<'callback>,
        F: 'static + Clone + Send + Sync + Fn(&'callback Lua, A) -> Result<R, Error>,
    {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                _handler.lock().unwrap().post(RawFunc::new(move || {
                    let lua = lua.lock().unwrap();
                    let _ = Self::def_fn_with_name(&lua, func.clone(), key);
                }));
            }
            None => {
                let lua = self.lua.lock().unwrap();
                Self::def_fn_with_name(&lua, func.clone(), key)?;
            }
        }

        Ok(())
    }
    #[inline]
    pub fn load<'lua>(
        lua: &'lua Lua,
        source: &str,
        name: Option<&str>,
    ) -> Result<Function<'lua>, Error> {
        let vm = lua;
        Ok(vm.load(source, name)?)
    }
    pub fn load_nowait(
        &self,
        source: &'static str,
        name: Option<&'static str>,
    ) -> Result<(), Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                _handler.lock().unwrap().post(RawFunc::new(move || {
                    let lua = lua.lock().unwrap();
                    let _ = Self::load(&lua, source, name);
                }));
            }
            None => {
                let lua = self.lua.lock().unwrap();
                Self::load(&lua, source, name)?;
            }
        }

        Ok(())
    }
    pub fn exec(
        &self,
        source: &'static str,
        name: Option<&'static str>,
    ) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(&_handler, move || {
                    Self::_exec(&lua, source, name)
                })
            }
            None => Self::_exec(&self.lua.clone(), source, name),
        }
    }
    pub fn exec_nowait(
        &self,
        source: &'static str,
        name: Option<&'static str>,
    ) -> Result<(), Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                _handler.lock().unwrap().post(RawFunc::new(move || {
                    let _ = Self::_exec(&lua.clone(), source, name);
                }));
            }
            None => {
                Self::_exec(&self.lua.clone(), source, name)?;
            }
        }

        Ok(())
    }
    #[inline]
    fn _exec(lua: &Arc<Mutex<Lua>>, source: &str, name: Option<&str>) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        Ok(vm.exec(source, name)?)
    }
    #[inline]
    pub fn exec_multi<'lua, R>(lua: &'lua Lua, source: &str, name: Option<&str>) -> Result<R, Error>
    where
        R: FromLuaMulti<'lua>,
    {
        let vm = lua;
        Ok(vm.exec(source, name)?)
    }
    pub fn eval(
        &self,
        source: &'static str,
        name: Option<&'static str>,
    ) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(&_handler, move || {
                    Self::_eval(&lua, source, name)
                })
            }
            None => Self::_eval(&self.lua.clone(), source, name),
        }
    }
    #[inline]
    fn _eval(lua: &Arc<Mutex<Lua>>, source: &str, name: Option<&str>) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        Ok(vm.eval(source, name)?)
    }
    #[inline]
    pub fn eval_multi<'lua, R>(lua: &'lua Lua, source: &str, name: Option<&str>) -> Result<R, Error>
    where
        R: FromLuaMulti<'lua>,
    {
        let vm = lua;
        Ok(vm.eval(source, name)?)
    }

    pub fn call(&self, name: &'static str, args: LuaMessage) -> Result<LuaMessage, Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                self.wait_async_lua_message_result(&_handler, move || {
                    Self::_call(&lua, name, args.clone())
                })
            }
            None => Self::_call(&self.lua.clone(), name, args),
        }
    }
    pub fn call_nowait(&self, name: &'static str, args: LuaMessage) -> Result<(), Error> {
        match self.handler.clone() {
            Some(_handler) => {
                let lua = self.lua.clone();
                _handler.lock().unwrap().post(RawFunc::new(move || {
                    let _ = Self::_call(&lua.clone(), name, args.clone());
                }));
            }
            None => {
                Self::_call(&self.lua.clone(), name, args)?;
            }
        }
        Ok(())
    }
    #[inline]
    fn _call(lua: &Arc<Mutex<Lua>>, name: &str, args: LuaMessage) -> Result<LuaMessage, Error> {
        let vm = lua.lock().unwrap();
        let func: Function = vm.globals().get::<_, Function>(name)?;

        Ok(func.call::<_, LuaMessage>(args)?)
    }
    #[inline]
    pub fn call_multi<'lua, A, R>(lua: &'lua Lua, name: &str, args: A) -> Result<R, Error>
    where
        A: ToLuaMulti<'lua> + Send + Sync + Clone + 'static,
        R: FromLuaMulti<'lua>,
    {
        let vm = lua;
        let func: Function = vm.globals().get::<_, Function>(name)?;

        Ok(func.call::<_, R>(args)?)
    }
}

#[test]
fn test_actor_new() {
    use rlua::Variadic;

    fn test_actor(act: Actor) {
        let _ = act.exec_nowait(
            r#"
            i = 1
        "#,
            None,
        );
        assert_eq!(Some(1), Option::from(act.get_global("i").ok().unwrap()));

        let v = act.eval(
            r#"
            3
        "#,
            None,
        );
        assert_eq!(Some(3), Option::from(v.ok().unwrap()));

        act.exec(
            r#"
            function testit (i)
                local Object = require("src.test")
                return Object:calc1(i)
            end
        "#,
            None,
        ).ok()
        .unwrap();
        match act.call("testit", LuaMessage::from(1)) {
            Ok(_v) => {
                assert_eq!(Some(2), Option::from(_v));
            }
            Err(_err) => {
                println!("{:?}", _err);
                panic!(_err);
            }
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
            Actor::def_fn_with_name(
                &vm,
                |_, strings: Variadic<String>| {
                    // (This is quadratic!, it's just an example!)
                    Ok(strings.iter().fold("".to_owned(), |a, b| a + b))
                },
                "join",
            ).ok()
            .unwrap();
            assert_eq!(
                vm.eval::<bool>(r#"check_equal({"a", "b", "c"}, {"a", "b", "c"})"#, None)
                    .ok()
                    .unwrap(),
                true
            );
            assert_eq!(
                vm.eval::<bool>(r#"check_equal({"a", "b", "c"}, {"d", "e", "f"})"#, None)
                    .ok()
                    .unwrap(),
                false
            );
            assert_eq!(
                vm.eval::<String>(r#"join("a", "b", "c")"#, None)
                    .ok()
                    .unwrap(),
                "abc"
            );
        }

        act.set_global(
            "arr1",
            LuaMessage::from(vec![LuaMessage::from(1), LuaMessage::from(2)]),
        ).ok()
        .unwrap();

        let v = Option::<Vec<LuaMessage>>::from(act.get_global("arr1").ok().unwrap());
        assert_eq!(LuaMessage::from(1), v.clone().unwrap()[0]);
        assert_eq!(LuaMessage::from(2), v.clone().unwrap()[1]);
        assert_eq!(2, v.clone().unwrap().len());
    }

    let _ = test_actor(Actor::new_with_handler(None));
    let _ = test_actor(Actor::new());
}
