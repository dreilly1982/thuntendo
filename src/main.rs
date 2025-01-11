mod nes;
mod cpu;
mod cartridge;
mod mappers;
mod ppu;
mod apu;
use nes::NES;
use std::env;

use pixels::{Error, Pixels, SurfaceTexture};
use std::time::{Duration, Instant};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType},
};

use std::io::{stdout, Write};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;



fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let fname = &args[1];
    let bus = NES::new();
    let mut system_counter: u32 = 0;
    bus.insert_cartridge(fname);

    

    bus.reset(&mut system_counter);

    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 *3.0);
        WindowBuilder::new()
            .with_title("Thuntendo")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels: Pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let target_frame_time = Duration::from_secs_f64(1.0/60.0);
    let mut last_frame_time = Instant::now();
    // bus.set_sample_frequency(44100);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if let KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(key),
                        ..
                    } = input
                    {
                        match key {
                            VirtualKeyCode::Escape => {
                                *control_flow = ControlFlow::Exit;
                            }
                            VirtualKeyCode::Q => {
                                *control_flow = ControlFlow::Exit;
                            }
                            VirtualKeyCode::X => {
                                bus.controller.borrow_mut()[0] |= 0x80;
                            }
                            VirtualKeyCode::Z => {
                                bus.controller.borrow_mut()[0] |= 0x40;
                            }
                            VirtualKeyCode::A => {
                                bus.controller.borrow_mut()[0] |= 0x20;
                            }
                            VirtualKeyCode::S => {
                                bus.controller.borrow_mut()[0] |= 0x10;
                            }
                            VirtualKeyCode::Up => {
                                bus.controller.borrow_mut()[0] |= 0x08;
                            }
                            VirtualKeyCode::Down => {
                                bus.controller.borrow_mut()[0] |= 0x04;
                            }
                            VirtualKeyCode::Left => {
                                bus.controller.borrow_mut()[0] |= 0x02;
                            }
                            VirtualKeyCode::Right => {
                                bus.controller.borrow_mut()[0] |= 0x01;
                            }
                            _ => (),
                        }
                    } else if let KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(key),
                        ..
                    } = input
                    {
                        match key {
                            VirtualKeyCode::X => {
                                bus.controller.borrow_mut()[0] &= 0x7F;
                            }
                            VirtualKeyCode::Z => {
                                bus.controller.borrow_mut()[0] &= 0xBF;
                            }
                            VirtualKeyCode::A => {
                                bus.controller.borrow_mut()[0] &= 0xDF;
                            }
                            VirtualKeyCode::S => {
                                bus.controller.borrow_mut()[0] &= 0xEF;
                            }
                            VirtualKeyCode::Up => {
                                bus.controller.borrow_mut()[0] &= 0xF7;
                            }
                            VirtualKeyCode::Down => {
                                bus.controller.borrow_mut()[0] &= 0xFB;
                            }
                            VirtualKeyCode::Left => {
                                bus.controller.borrow_mut()[0] &= 0xFD;
                            }
                            VirtualKeyCode::Right => {
                                bus.controller.borrow_mut()[0] &= 0xFE;
                            }
                            _ => (),
                        }
                    }
                }
                    
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                if let Err(e) = pixels.render() {
                    eprintln!("pixels.render() failed: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
                bus.reset_frame_complete();
            },
            Event::MainEventsCleared => {
                let now = Instant::now();
                let elapsed = now - last_frame_time;

                let samples_per_frame = 44100 / 60;
                let mut samples = 0;

                // if !bus.cpu_complete() {
                //     window.request_redraw();
                // } else {
                //     *control_flow = ControlFlow::Exit;
                // }

                while !bus.get_frame_complete() {
                    bus.clock(pixels.frame_mut(), &mut system_counter);
                }

                if elapsed >= target_frame_time {
                    // println!("Elapsed: {:?}", elapsed);
                    window.request_redraw();
                    last_frame_time = now;
                } else {
                    *control_flow = ControlFlow::WaitUntil(now + target_frame_time - elapsed);
                }

                // while samples <= samples_per_frame {
                //     bus.clock(pixels.frame_mut(), &mut system_counter);
                //     if *bus.audio_sample_ready.borrow() {
                //         samples += 1;
                //     }
                    
                // }

                // while !bus.clock(pixels.frame_mut(), &mut system_counter) { }
                println!("{:?}", elapsed);
                last_frame_time = now;
                window.request_redraw();
            },
            _ => ()
        }
    });
}
