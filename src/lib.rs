// We do this since rust complains about everything being FFI unsafe, yet it doesn't really matter
// since we are USUALLY having rust shared libs.
#![allow(improper_ctypes_definitions)]

use std::sync::{Arc, Mutex};
use dlauncher::extension::{ExtensionContext, ExtensionExitCode};
use dlauncher::extension::response::{ExtensionResponse, ExtensionResponseIcon};
use dlauncher::util::init_logger;
use lazy_static::lazy_static;

// Used so we can run stuff in static
lazy_static! {
  // Here we have a PREFIX variable which is a String wrapped in a Arc and Mutex.
  // We need this to be thread safe and we need it mutable.
  // Usually in the on_init function is where this variable would be set.
  #[derive(Debug)] static ref PREFIX_TERMINAL: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
  #[derive(Debug)] static ref PREFIX_RUN: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
  #[derive(Debug)] static ref TERMINAL: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

// No mangle is required since when we access this function from dlauncher, we want the same function
// name to be used.
// This function is used at the startup of Dlauncher, and only runs once.
#[no_mangle]
pub unsafe extern "C" fn on_init(ctx: ExtensionContext) -> ExtensionExitCode {
  init_logger();

  // we can use the ExtensionContext if we want to access the extensions config
  // to get maybe a prefix, or other user changeable thing.
  if ctx.config.contains_key("prefix_terminal") == false {ctx.config.set("prefix_terminal", ">");}
  if ctx.config.contains_key("prefix_run") == false {ctx.config.set("prefix_run", "!");}
  if ctx.config.contains_key("terminal") == false {ctx.config.set("terminal", "alacritty --hold -e");}
  let prefix_terminal = ctx.config.get("prefix_terminal").unwrap();
  let prefix_run = ctx.config.get("prefix_run").unwrap();
  let terminal_command = ctx.config.get("terminal").unwrap();
  let mut p = PREFIX_TERMINAL.lock().unwrap();
  let mut p2 = PREFIX_RUN.lock().unwrap();
  let mut term = TERMINAL.lock().unwrap();
  *p = prefix_terminal;
  *p2 = prefix_run;
  *term = terminal_command;

  // Return Ok since no errors occurred during this.
  ExtensionExitCode::Ok
}

// This function is called whenever the input is changed in the ui.
#[no_mangle]
//pub unsafe extern "C" fn on_input(ctx: ExtensionContext) -> ExtensionExitCode {
pub unsafe extern "C" fn on_input(ctx: ExtensionContext){
  // This call is necessary if we want to do anything relating to GTK/GDK.
  gtk::set_initialized();

  if let Some(ref input) = ctx.input {
    // It will check if the inputs prefix is the same as the one we stored in the PREFIX variable.
    // Since we want this to "silently error out" we just return Ok instead of an Error.
    // We also check if the query is empty here.
    // Below:
    let p = &*PREFIX_TERMINAL.lock().unwrap();
    let p2 = &*PREFIX_RUN.lock().unwrap();
    let term = &*TERMINAL.lock().unwrap();
    let user_input = ctx.input.as_ref().expect("failed to read input").query().to_string();
    //let command = String::new();
    
    if input.prefix() == p {
        //log::info!("matched {}", p);
        let command = format!("{} {}", term, user_input);
        //log::info!("input is: {}", user_input);
        let response_term = ExtensionResponse::builder(&ctx.name)
        .line_on_enter(
            "Run command in terminal",
            "",
            ExtensionResponseIcon::themed("utilities-terminal"),
        move |_| {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .spawn()
                .expect("terminal command failed to start");
        }
        )
        .build(ctx.window.clone());
        ctx.window.show_results(response_term, true);
        //return ExtensionExitCode::Ok;
    } else if input.prefix() == p2 {
        let command = user_input;
        let response = ExtensionResponse::builder(&ctx.name)
            .line_on_enter(
                "Run command",
                "",
                ExtensionResponseIcon::themed("utilities-terminal"),
                move |_| {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&command)
                        .spawn()
                        .expect("command failed to start");
                })
        .build(ctx.window.clone());
        ctx.window.show_results(response, true);
    } else {
        return;
    }
  }
}
