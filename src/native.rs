use std::time::SystemTime;

use crate::{
    environment::EnvRef,
    tokens::{Callable, Literal, LoxCallable},
};

pub(crate) fn define_native_functions(env_ref: EnvRef) {
    define_clock(env_ref)
}

fn define_clock(mut env_ref: EnvRef) {
    env_ref.define(
        "clock",
        Literal::Callable(LoxCallable::new(
            "clock".to_string(),
            Callable::Native(|| {
                let now = SystemTime::now();
                let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
                Literal::Number(duration.as_secs_f64())
            }),
        )),
    );
}
