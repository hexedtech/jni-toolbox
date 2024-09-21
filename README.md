# jni-macro
this is a simple procedural macro crate to automatically generate JNI-compatible extern functions

it also wraps functions returning `Result<>`, making short-circuiting easy

## usage
just annotate your functions with

```rust
#[jni_macro::jni(package = "your.package.path", class = "ContainerClass")]
fn your_function_name(arg: i32) -> Result<(), String> {
	// your code here
}
```

by specifying package and class, this crate will write an appropriate wrapper with the right function name. If inner function returns a `Result<>`, wrapper will also handle it. (currently just panics, soon will throw exceptions!)

## examples
the following function:
```rust
#[java_easy_jni::jni(package = "mp.code", class = "Client")]
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
