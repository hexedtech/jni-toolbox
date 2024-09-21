# jni-toolbox
this is a simple crate built around [jni-rs](https://github.com/jni-rs/jni-rs) to automatically generate JNI-compatible extern functions

it also wraps functions returning `Result<>`, making short-circuiting easy

## usage
just specify package and class on your function, and done!

```rust
#[jni_toolbox::jni(package = "your.package.path", class = "ContainerClass")]
fn your_function_name(arg: String) -> Result<Vec<String>, String> {
  Ok(arg.split('/').map(|x| x.to_string()).collect())
}
```

every type that must go into/from Java must implement `IntoJava` or `FromJava` (methods will receive a `&mut JNIEnv` and can return errors).
most primitives already have them implemented. conversions are automatic and the wrapper function will invoke IntoJava/FromJava for every type,
passing an environment reference.


Errors are thrown automatically when a `Result` is an error. For your errors to work, you must implement the `JniToolboxError` trait for your errors,
(which just returns the path to your Java error class) and then make a Java error wrapper which can be constructed with a single string argument.
functions returning `Result`s will automatically have their return value unwrapped and, if is an err, throw an exception and return early.

to throw simple exceptions, it's possible to use the `exception` attribute. just pass your exception's path (must be constructable with a single string argument!)

to return pointer type values, add the `ptr` attribute


## examples
the following function:
```rust
#[jni(package = "mp.code", class = "Client", ptr)]
fn connect(config: Config) -> Result<Client, ConnectionError> {
  tokio().block_on(Client::connect(config))
}
```

gets turned into these two functions:

<details><summary>show macro expansion</summary>

```rust
fn connect(config: Config) -> Result<Client, ConnectionError> {
  tokio().block_on(Client::connect(config))
}

#[no_mangle]
#[allow(unused_mut)]
pub extern "system" fn Java_mp_code_Client_connect<'local>(
  mut env: jni::JNIEnv<'local>,
  _class: jni::objects::JClass<'local>,
  mut config: <Config as jni_toolbox::FromJava<'local>>::T,
) -> <Client as jni_toolbox::IntoJava<'local>>::T {
  use jni_toolbox::{FromJava, IntoJava, JniToolboxError};
  let mut env_copy = unsafe { env.unsafe_clone() };
  let config_new = match jni_toolbox::from_java_static::<Config>(&mut env, config) {
    Ok(x) => x,
    Err(e) => {
      let _ = env.throw_new(
        "java/lang/RuntimeException",
        $crate::__export::must_use({
          let res = $crate::fmt::format($crate::__export::format_args!("{e:?}"));
          res
        }),
      );
      return std::ptr::null_mut();
    }
  };
  match connect(config_new) {
    Err(e) => match env_copy.find_class(e.jclass()) {
      Err(e) => {
        $crate::panicking::panic_fmt($crate::const_format_args!(
          "error throwing Java exception -- failed resolving error class: {e}"
        ));
      }
      Ok(class) => match env_copy.new_string($crate::__export::must_use({
        let res = $crate::fmt::format($crate::__export::format_args!("{e:?}"));
        res
      })) {
        Err(e) => {
          $crate::panicking::panic_fmt($crate::const_format_args!(
            "error throwing Java exception --  failed creating error string: {e}"
          ));
        }
        Ok(msg) => match env_copy.new_object(
          class,
          "(Ljava/lang/String;)V",
          &[jni::objects::JValueGen::Object(&msg)],
        ) {
          Err(e) => {
            $crate::panicking::panic_fmt($crate::const_format_args!(
              "error throwing Java exception -- failed creating object: {e}"
            ));
          }
          Ok(obj) => match env_copy.throw(jni::objects::JThrowable::from(obj)) {
            Err(e) => {
              $crate::panicking::panic_fmt($crate::const_format_args!(
                "error throwing Java exception -- failed throwing: {e}"
              ));
            }
            Ok(_) => return std::ptr::null_mut(),
          },
        },
      },
    },
    Ok(ret) => match ret.into_java(&mut env_copy) {
      Ok(fin) => return fin,
      Err(e) => {
        let _ = env_copy.throw_new(
          "java/lang/RuntimeException",
          $crate::__export::must_use({
            let res = $crate::fmt::format($crate::__export::format_args!("{e:?}"));
            res
          }),
        );
        return std::ptr::null_mut();
      }
    },
  }
}
```

</details>


## status
this crate is rather early and intended mostly to maintain [`codemp`](https://github.com/hexedtech/codemp) java bindings, however it's also quite small and only runs at comptime, so should be rather safe to use
