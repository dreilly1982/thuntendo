mod nes;
mod cpu;
mod cartridge;
mod mappers;
mod ppu;
mod apu;
mod utils;
use nes::NES;
use std::{env, sync::{Arc, Mutex}};
use utils::consts::{HEIGHT, WIDTH};

use pixels::{Error, Pixels, PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use ringbuf::HeapRb as RingBuffer;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let fname = &args[1];
    let bus = NES::new();
    let event_loop = EventLoop::new();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let config = device.default_output_config().expect("no default output config available").config();

    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();

    println!("Default output config: {:?}", config);

    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    let ring_buffer = RingBuffer::<f32>::new(2048); // Adjust size as needed
    let (mut producer, mut consumer) = ring_buffer.split();

    std::thread::spawn(move || {
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels as usize) {
                    let ouput = consumer.pop().unwrap_or(0.0);
                    for (_, sample) in frame.iter_mut().enumerate() {
                        *sample = ouput;
                    }
                }
            },
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
            },
            None,
        ).unwrap();

        stream.play().unwrap();

        while *running_clone.lock().unwrap() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

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
        PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .enable_vsync(false)
            .build()?
    };

    bus.set_sample_frequency(sample_rate as u32);

    bus.insert_cartridge(fname);
    bus.reset();


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
                            VirtualKeyCode::R => {
                                bus.reset();
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

                while !bus.clock(pixels.frame_mut(), &mut producer) {}

                if bus.get_frame_complete() {
                    window.request_redraw();
                }        
            },
            _ => ()
        }
    });
}
