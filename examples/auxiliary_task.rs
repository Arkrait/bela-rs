//! Produces a sine wave while printing "this is a string" repeatedly,
//! appending "LOL" to every iteration.
//! 
extern crate bela;
extern crate sample;

use std::{thread, time};
use bela::*;

#[derive(Clone)]
struct PrintTask<F> {
    callback: F,
    args: String,
}

impl<F> Auxiliary for PrintTask<F>
where F: FnMut(&mut String),
      for<'r> F: FnMut(&'r mut String)
{
    type Args = String;

    fn destructure(&mut self) -> (&mut FnMut(&mut String), &mut Self::Args) {
        let PrintTask {
            callback,
            args,
        } = self;

        (callback, args)
    }
}

struct MyData<'a> {
    frame_index: usize,
    tasks: Vec<CreatedTask<'a>>
}

type BelaApp<'a> = Bela<AppData<'a, MyData<'a>>>;

fn main() {
    go().unwrap();
}

fn go() -> Result<(), error::Error> {
    let what_to_print = "this is a string".to_string();
    let print_task = PrintTask {
        callback: |args: &mut String| {
            args.push_str("LOL");
            println!("{}", args);
        },
        args: what_to_print,
    };

    let more_to_print = "this is another string".to_string();
    let mut another_print_task = PrintTask {
        callback: |args: &mut String| {
            args.push_str("LOL");
            println!("{}", args);
        },
        args: more_to_print,
    };

    let mut boxed = Box::new(print_task);

    let mut setup = |_context: &mut Context, user_data: &mut MyData| -> Result<(), error::Error> {
        println!("Setting up");
        user_data.tasks.push(BelaApp::create_auxiliary_task(&mut boxed, 10, "printing_stuff"));
        user_data.tasks.push(BelaApp::create_auxiliary_task(&mut another_print_task, 10, "printing_more_stuff"));
        Ok(())
    };

    let mut cleanup = |_context: &mut Context, _user_data: &mut MyData| {
        println!("Cleaning up");
    };

    let mut render = |_context: &mut Context, user_data: &mut MyData| {
        if user_data.frame_index % 1024 == 0 {
            for task in user_data.tasks.iter() {
                BelaApp::schedule_auxiliary_task(task);
            }
        }

        user_data.frame_index = user_data.frame_index.wrapping_add(1);
    };

    let my_data = MyData {
        tasks: Vec::new(),
        frame_index: 0,
    };

    let user_data = AppData::new(my_data, &mut render, Some(&mut setup), Some(&mut cleanup));

    let mut bela_app: Bela<AppData<MyData>> = Bela::new(user_data);
    let mut settings = InitSettings::default();
    bela_app.init_audio(&mut settings)?;
    bela_app.start_audio()?;

    while !bela_app.should_stop() {
        thread::sleep(time::Duration::new(1, 0));
    }

    bela_app.stop_audio();
    bela_app.cleanup_audio();

    Ok(())
}
