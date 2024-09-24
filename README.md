# jni-toolbox
[![Actions Status](https://github.com/hexedtech/jni-toolbox/actions/workflows/test.yml/badge.svg)](https://github.com/hexedtech/jni-toolbox/actions)
[![Crates.io Version](https://img.shields.io/crates/v/jni-toolbox)](https://crates.io/crates/jni-toolbox)
[![docs.rs](https://img.shields.io/docsrs/jni-toolbox)](https://docs.rs/jni-toolbox)

This is a simple crate built around [jni-rs](https://github.com/jni-rs/jni-rs) to automatically generate JNI-compatible extern functions.

It also wraps functions returning `Result<>`, making short-circuiting easy.

## Usage
Just specify package and class on your function, and done!

```rust
#[jni_toolbox::jni(package = "your.package.path", class = "ContainerClass")]
fn your_function_name(arg: String) -> Result<Vec<String>, String> {
  Ok(arg.split('/').map(|x| x.to_string()).collect())
}
```

### Conversions
Every type that is meant to be sent to Java must implement `IntoJavaObject` (or, unlikely, `IntoJavaPrimitive`); every type that is meant to be
received from Java must implement `FromJava`. Most primitives and a few common types should already be implemented.

```rust
impl<'j> IntoJavaObject for MyClass {
  type T = jni::objects::JObject<'j>
  fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
    let hello = env.new_string("world")?;
    // TODO!!
  }
}
```

### Pointers
Note that, while it is possible to pass raw pointers to the JVM, it is not safe by default and must be done with extreme care.

### Exceptions
Errors are thrown automatically when a `Result` is an error. For your errors to work, you must implement the `JniToolboxError` trait for your errors,
(which just returns the path to your Java error class) and then make a Java error wrapper which can be constructed with a single string argument.

Functions returning `Result`s will automatically have their return value unwrapped and, if is an err, throw an exception and return early.

```rust
impl JniToolboxError for MyError {
  fn jclass(&self) -> String {
    "my/package/some/MyError".to_string()
  }
}
```

```java
package my.package.some;
public class MyError extends Throwable {
  public MyError(String x) {
    // TODO
  }
}
```

To throw simple exceptions, it's possible to use the `exception` attribute. Pass the exception's fully qualified name (must have a constructor
that takes in a single `String` argument).

### Examples
The following function:
```rust
#[jni(package = "mp.code", class = "Client", ptr)]
fn connect(config: Config) -> Result<Client, ConnectionError> {
  super::tokio().block_on(Client::connect(config))
}
```

generates a matching expanded function invoking it:

```rust
fn connect(config: Config) -> Result<Client, ConnectionError> {
  super::tokio().block_on(Client::connect(config))
}

#[no_mangle]
#[allow(unused_unit)]
pub extern "system" fn Java_mp_code_Client_connect<'local>(
  mut env: jni::JNIEnv<'local>,
  _class: jni::objects::JClass<'local>,
  config: <Config as jni_toolbox::FromJava<'local>>::From,
) -> <Client as jni_toolbox::IntoJava<'local>>::Ret {
  use jni_toolbox::{FromJava, IntoJava, JniToolboxError};
  let config_new = match jni_toolbox::from_java_static::<Config>(&mut env, config) {
    Ok(x) => x,
    Err(e) => {
      let _ = env.throw_new("java/lang/RuntimeException", format!("{e:?}"));
      return std::ptr::null_mut();
    }
  };
  let mut env_copy = unsafe { env.unsafe_clone() };
  let result = connect(config_new);
  let ret = match result {
    Ok(x) => x,
    Err(e) => match env_copy.find_class(e.jclass()) {
      Err(e) => panic!("error throwing Java exception -- failed resolving error class: {e}"),
      Ok(class) => match env_copy.new_string(format!("{e:?}")) {
        Err(e) => panic!("error throwing Java exception --  failed creating error string: {e}"),
        Ok(msg) => match env_copy.new_object(class, "(Ljava/lang/String;)V", &[jni::objects::JValueGen::Object(&msg)]) {
          Err(e) => panic!("error throwing Java exception -- failed creating object: {e}"));
          Ok(obj) => match env_copy.throw(jni::objects::JThrowable::from(obj)) {
            Err(e) => panic!("error throwing Java exception -- failed throwing: {e}"),
            Ok(_) => return std::ptr::null_mut(),
          },
        },
      },
    },
  };
  match ret.into_java(&mut env_copy) {
    Ok(fin) => fin,
    Err(e) => {
      let _ = env_copy.throw_new("java/lang/RuntimeException", format!("{e:?}"));
      std::ptr::null_mut()
    }
  }
}
```

## Status
This crate is early and intended mostly to maintain [`codemp`](https://github.com/hexedtech/codemp)'s Java bindings, so things not used
there may be missing or slightly broken. However, the crate is also quite small and only runs at compile time, so trying it out in your
own project should not be a problem.
