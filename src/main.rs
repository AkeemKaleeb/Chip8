use std::env;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;

mod chip8;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() -> Result<(), String> {
    // Command Line arguments: Usage: cargo run <rom_path>
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Error Usage: {} <rom_path>", args[0]);
        std::process::exit(1);
    }

    let mut chip8 = chip8::Chip8::new();
    let _ = chip8.load_rom(&args[1]);

    // Video Render
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Chip8 Emu", (WIDTH * 10) as u32, (HEIGHT * 10) as u32)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                _ => {}
            }
        }
        chip8.cycle();

        if chip8.draw_flag {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let idx = x + y * WIDTH;
                    if chip8.display[idx] == 1 {
                        canvas.set_draw_color(Color::RGB(255, 255, 255));
                    }
                    else {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                    }
                    canvas.fill_rect(Rect::new((x * 10) as i32, (y * 10) as i32, 10, 10)).unwrap();
                }
            }

            chip8.draw_flag = false;
            canvas.present();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }


    Ok(())
}