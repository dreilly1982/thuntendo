mod nes;
mod cpu;
mod cartridge;
mod mappers;
mod ppu;
mod apu;
mod utils;
use nes::NES;
use std::{any, env, sync::{Arc, Mutex}, thread};
use utils::consts::{WIDTH, HEIGHT, NTSC_CRYSTAL_FREQUENCY};

use pixels::{Error, Pixels, PixelsBuilder, SurfaceTexture};
use std::time::{Duration, Instant};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, SizedSample, StreamConfig};
use cpal::{Sample, SampleFormat};

use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType},
};

use ringbuf::HeapRb as RingBuffer;

use std::io::{stdout, Write};


fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let fname = &args[1];
    let bus = NES::new();
    let mut system_counter: u32 = 0;
    let event_loop = EventLoop::new();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let config = device.default_output_config().expect("no default output config available").config();

    let sample_rate = config.sample_rate.0 as f32;
    let audio_time_per_system_sample = 1.0 / sample_rate;
    let audio_time_per_nes_clock = 1.0 / NTSC_CRYSTAL_FREQUENCY;

    let audio_sample_count = Arc::new(Mutex::new(0usize));
    let nes_clock_count = Arc::new(Mutex::new(0usize));

    let audio_sample_count_clone = Arc::clone(&audio_sample_count);

    let ring_buffer = RingBuffer::<f32>::new(4096); // Adjust size as needed
    let (mut producer, mut consumer) = ring_buffer.split();


    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let mut sample_count = audio_sample_count.lock().unwrap();
            for sample in data.iter_mut() {
                *sample = consumer.pop().unwrap_or(0f32);
                *sample_count += 1;
            }
        },
        move |err| {
            eprintln!("an error occurred on stream: {}", err);
        },
        None,
    ).unwrap();

    stream.play().unwrap();

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
        // Pixels::new(WIDTH, HEIGHT, surface_texture)?
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .enable_vsync(false)
            .build()?
    };

    bus.set_sample_frequency( config.sample_rate.0);

    bus.insert_cartridge(fname);
    bus.reset(&mut system_counter);

    let last_frame_time = Arc::new(Mutex::new(Instant::now()));

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
                // sample_sender.send(*bus.audio_sample.borrow() as f32).unwrap();
            },
            Event::MainEventsCleared => {

                let now = Instant::now();
                let mut last_time = last_frame_time.lock().unwrap();
                let elapsed = now.duration_since(*last_time).as_secs_f32();
                *last_time = now;

                let mut nes_clock_count = nes_clock_count.lock().unwrap();
                *nes_clock_count += (elapsed / audio_time_per_nes_clock) as usize;

                let audio_sample_count = audio_sample_count_clone.lock().unwrap();
                let elapsed_audio_time = *audio_sample_count as f32 * audio_time_per_system_sample;
                let elapsed_emulator_time = *nes_clock_count as f32 * audio_time_per_nes_clock;

                if elapsed_emulator_time > elapsed_audio_time {
                    let sleep_time = elapsed_emulator_time - elapsed_audio_time;
                    thread::sleep(Duration::from_secs_f32(sleep_time));
                }

                while !bus.get_frame_complete() {
                    bus.clock(pixels.frame_mut(), &mut system_counter, &mut producer);
                }

                window.request_redraw();
                
            },
            _ => ()
        }
    });
}
