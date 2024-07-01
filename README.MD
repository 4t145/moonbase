# Moonbase

> Fly me to the moon, and let me play among the stars.

**Moonbase** is a meta framework which provides a **easy** way to create a **maintainable** service.

## What can Moonbase Provide?

It's basically a easy-to-use **state** for your web framework. And you can build your own business context based on a `AppContext` aka `Moonbase`.


### A way to get **RESOURCE**

If you are addicted to global static resource, moonbase can provide a alterative way to get app-wise unique resource by `MoonbaseResource` from anywhere in your program.

```rust
let AppName(app_name) = app.get_resource::<AppName>();
```
### A way to get **COMPONENT**

Ok, your resource may be a module-wise resource, a temporary state, or some random thing you want to share between tasks. You can store them as a `MoonbaseComponent`. And you can get the resource by a 2-tuple of `Type` and `&str`.


```rust
let mail_config = app.get_component::<MailConfig>(&module_name);
```

### A way to use **CONTEXT**

Get anything you want from a certain context in your code, as long as the thing can be **EXTRACTED** from the context.


You may handler request like this:
```rust
async fn handle_some_request(redis_client: RedisClient, http_client: HttpClient, config: Arc<MyModuleConfig>, db_connection: Db, ...) -> Result<(), Error> {
    ...
}
```

Just pass this handler to a certain context after you implemented `Extract<MyContext>` for them!
```rust
request_context.fallible_call(handle_some_request).await?;
```

### A way to manage **DAEMON**

Don't spawn your task **everywhere**, and then you don't know their status at all. Instead, let them be managed by our engineers in the **Moonbase**.

```rust
async fn run(app: &Moonbase) {
    // do some initializations
    app.run_daemon::<AxumWebService>().await?;
    app.run_daemon::<ListenConfigChangeService>().await?;
    app.run_daemon::<RunCronBotService>().await?;
    app.run_daemon::<DoSomeRandomThingService>().await?;
    let handle = app.run_daemon::<ListenShutdownSignal>().await?;
    handle.wait().await;
}
```

### A way to share **SIGNAL**
```rust
pub struct MySignalSymbol;
async fn async_main() -> anyhow::Result<()> {
    moonbase.set_signal(SignalKey::from_type::<MySignalSymbol>(), Signal::new());
    let signal = moonbase.get_signal(&SignalKey::symbol::<MySignalSymbol>()).unwrap();
    signal.get_sender().send();
    signal.recv().await;
    Ok(())
}
```
