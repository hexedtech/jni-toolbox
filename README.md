# jni-toolbox
this is a simple crate built around [jni-rs](https://github.com/jni-rs/jni-rs) to automatically generate JNI-compatible extern functions

it also wraps functions returning `Result<>`, making short-circuiting easy

## usage
you must implement `JniToolboxError` trait for your errors, so that they can be converted to Java errors

you will need to define classes for them, and implement `JniToolboxError` returning the class path

alternatively, an `exception` class can be specified with the `exception` attribute

then just annotate your functions with

```rust
#[jni_toolbox::jni(package = "your.package.path", class = "ContainerClass")]
fn your_function_name(arg: i32) -> Result<(), String> {
	// your code here
}
```

by specifying package and class, this crate will write an appropriate wrapper with the right function name. If inner function returns a `Result<>`, wrapper will also handle it. (currently just panics, soon will throw exceptions!)

note that input/output arguments must be natively FFI safe: there will be no hidden translations! you will have to un-marshal strings yourself

## examples
the following function:
```rust
#[jni_toolbox::jni(package = "mp.code", class = "Client")]
fn connect(env: JNIEnv, cacca: JString) -> Result<(), ConnectionError> {
  let config = codemp::api::Config::new("asd".into(), "dsa".into());
  tokio().block_on(codemp::Client::connect(config))?;
  Ok(())
}
```

gets turned into this couple of functions:
```rust
fn connect(env: JNIEnv, host: JString) -> Result<(), ConnectionError> {
  let config = codemp::api::Config::new("mail@example.net".into(), "dont-use-this-password".into());
  tokio::runtime::current().block_on(codemp::Client::connect(config))?;
  Ok(())
}

#[no_mangle]
pub extern "system" fn Java_mp_code_Client_connect<'local>(env: JNIEnv, host: JString) -> () {
  match connect(env, cacca) {
    Ok(x) => x,
    Err(e) => {
      $crate::panicking::panic_fmt($crate::const_format_args!("error in JNI!"));
    }
  }
}
```


## status
this crate is rather early and intended mostly to maintain [`codemp`](https://github.com/hexedtech/codemp) java bindings, however it's also quite small and only runs at comptime, so should be rather safe to use
