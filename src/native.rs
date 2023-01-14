use std::time::SystemTime;

use rust_decimal::{prelude::FromPrimitive, Decimal};

use crate::{
    environment::Environment,
    tokens::{Callable, Literal, LoxCallable},
};

pub(crate) fn define_native_functions(env: Environment) {
    define_clock(env)
}

fn define_clock(mut env: Environment) {
    env.define(
        "clock",
        Literal::Callable(LoxCallable::new(
            "clock".to_string(),
            Callable::Native(|| {
                let now = SystemTime::now();
                let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();

                Literal::Number(Decimal::from_f64(duration.as_secs_f64()).unwrap())
            }),
        )),
    );
}
