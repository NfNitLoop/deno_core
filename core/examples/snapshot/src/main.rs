use std::{env::current_dir, rc::Rc};

use deno_core::{
  error::AnyError, extension, op2, FsModuleLoader, JsRuntime,
  PollEventLoopOptions, RuntimeOptions,
};

fn main() {
  let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();
  if let Err(error) = runtime.block_on(run_js("./example.js")) {
    eprintln!("error: {}", error);
  }
}

#[op2(fast)]
fn op_call_rust(#[string] value: String) {
  println!("Received this value from JS: {value}");
}

extension!(runjs_extension, ops = [op_call_rust,],);

async fn run_js(file_path: &str) -> Result<(), AnyError> {
  let cwd = current_dir()?;
  let main_module = deno_core::resolve_path(file_path, &cwd)?;

  let mut js_runtime = JsRuntime::new(RuntimeOptions {
    module_loader: Some(Rc::new(FsModuleLoader)),
    startup_snapshot: Some(RUNTIME_SNAPSHOT),
    extensions: vec![runjs_extension::init_ops()],
    ..Default::default()
  });

  let mod_id = js_runtime.load_main_es_module(&main_module).await?;
  let result = js_runtime.mod_evaluate(mod_id);
  js_runtime
    .run_event_loop(PollEventLoopOptions::default())
    .await?;
  result.await
}

// Load the snapshot generated by build.rs:
static RUNTIME_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/RUNJS_SNAPSHOT.bin"));
