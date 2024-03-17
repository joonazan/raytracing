use std::iter::{once, repeat};
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod glass;
mod refraction;
mod test_pattern;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("raytracing preview", N as u32, N as u32)
        .position_centered()
        .build()
        .unwrap();

    let sample_schedule = once(1).chain((1..=10).map(|x| 1 << x));
    let mut jobs = sample_schedule.flat_map(|s| (0..N).zip(repeat(s)));
    let mut jobs_finished = 0;
    let scene = glass::Scene::new();
    let mut hdr_image = [[0.; N]; N];

    const THREADS: usize = 16;
    let (tx, ready_rows) = sync_channel(THREADS);

    thread::scope(|rendering| {
        let mut threads_still_running = THREADS;
        let start_time = Instant::now();

        for (y, samples) in jobs.by_ref().take(THREADS) {
            let tx = tx.clone();
            let scene = &scene;
            rendering.spawn(move || {
                tx.send(render_row(y, scene, samples)).unwrap();
            });
        }

        let mut event_pump = sdl_context.event_pump().unwrap();
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            if let Ok((mut y, mut row_colors)) = ready_rows.recv_timeout(Duration::from_millis(20))
            {
                let mut window_surface = window.surface(&event_pump).unwrap();
                window_surface.with_lock_mut(|colors| loop {
                    if jobs_finished < N {
                        hdr_image[y].copy_from_slice(&row_colors);
                    } else {
                        for x in 0..N {
                            hdr_image[y][x] /= 2.;
                            hdr_image[y][x] += row_colors[x] / 2.;
                        }
                    }
                    jobs_finished += 1;

                    for x in 0..N {
                        for channel in 0..3 {
                            colors[4 * N * y + 4 * x + channel] =
                                (hdr_image[y][x].powf(0.45) * 255.) as u8;
                        }
                    }

                    if let Some((job, samples)) = jobs.next() {
                        let tx = tx.clone();
                        let scene = &scene;
                        rendering.spawn(move || tx.send(render_row(job, scene, samples)).unwrap());
                    } else {
                        threads_still_running -= 1;
                        if threads_still_running == 0 {
                            println!(
                                "Rendering finished in {:.2} seconds.",
                                (Instant::now() - start_time).as_secs_f32()
                            );
                        }
                    }

                    if let Ok(x) = ready_rows.try_recv() {
                        (y, row_colors) = x
                    } else {
                        break;
                    }
                });
                window_surface.finish().unwrap();
            }
        }
    })
}

const N: usize = 1024;

trait Image {
    fn render(&self, x: usize, y: usize) -> f32;
}

fn render_row<S: Image>(y: usize, scene: &S, samples: u32) -> (usize, [f32; N]) {
    let mut output = [0.; N];

    for x in 0..N {
        for _ in 0..samples {
            output[x] += scene.render(x, y) / samples as f32;
        }
    }

    (y, output)
}
