use ctx::appenv::AppEnvironment;

mod ctx;
mod utils;

fn main() {
    let app_env: &AppEnvironment = AppEnvironment::get();
}
