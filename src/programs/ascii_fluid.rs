use crate::{print, println};
const WIDTH: usize = 80;
const HEIGHT: usize = 24;

const GRAVITY: f32 = 1.;
const PRESSURE: f32 = 4.;
const VISCOSITY: f32 = 7.;

const CHARS: [char; 13] = [
    ' ', '\'', '`', '-', '.', '|', '/', ',', '\\', '_', '\\', '/', '#',
];

fn sqrt(x: f32) -> f32 {
    let i = x.to_bits();
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    let res = y * (1.5 - 0.5 * x * y * y);
    1. / res
}

pub fn main() -> ! {
    let _x_sandbox_area_scan = 0;
    let _y_sandbox_area_scan = 0;
    let mut x_pos = [0 as isize; 2 * HEIGHT * WIDTH];
    let mut y_pos = [0 as isize; 2 * HEIGHT * WIDTH];
    let mut x_force = [0.; 2 * HEIGHT * WIDTH];
    let mut y_force = [0.; 2 * HEIGHT * WIDTH];
    let mut x_velocity = [0.; 2 * HEIGHT * WIDTH];
    let mut y_velocity = [0.; 2 * HEIGHT * WIDTH];
    let mut density = [0.; 2 * HEIGHT * WIDTH];
    let mut wall_flag = [0 as isize; 2 * HEIGHT * WIDTH];
    let mut total_of_particles = 0 as isize;
    let mut SCREENBUFFER = [[0 as u8; HEIGHT]; WIDTH];

    let mut particles_counter = 0;

    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            if i == 0 || j == 0 || i >= WIDTH - 2 || j >= HEIGHT - 3 {
                wall_flag[particles_counter] = 1;
                wall_flag[particles_counter + 1] = 1;
                total_of_particles += 2;
                x_pos[particles_counter] = i as isize;
                y_pos[particles_counter] = 2 * j as isize;
                x_pos[particles_counter + 1] = i as isize;
                y_pos[particles_counter + 1] = (2 * j + 1) as isize;
                particles_counter += 2;
            } else if i <= WIDTH / 2 && j >= 10 {
                wall_flag[particles_counter] = 0;
                wall_flag[particles_counter + 1] = 0;
                total_of_particles += 2;
                particles_counter += 2;
                x_pos[particles_counter] = i as isize;
                y_pos[particles_counter] = 2 * j as isize;
                x_pos[particles_counter + 1] = i as isize;
                y_pos[particles_counter + 1] = (2 * j + 1) as isize;
            }
        }
    }

    let mut x_particle_distance;
    let mut y_particle_distance;
    let _a = (3 as f32) * 9.;
    println!("{}", sqrt(100.));
    println!("Initialized!");
    loop {
        println!("Loop beginning");
        for particles_cursor in 0..total_of_particles {
            println!("{} {}", particles_cursor, 256 as f32 * 1.1);
            let _valeur = wall_flag[particles_cursor as usize];
            println!("mdr");
            //density[particles_cursor as usize] = valeur + 0.0001;

            for particles_cursor2 in 0..total_of_particles {
                println!("{}", particles_cursor2);
                x_particle_distance =
                    x_pos[particles_cursor as usize] - x_pos[particles_cursor2 as usize];
                y_particle_distance =
                    y_pos[particles_cursor as usize] - y_pos[particles_cursor2 as usize];
                let particles_distance = sqrt(
                    (x_particle_distance * x_particle_distance
                        + y_particle_distance * y_particle_distance) as f32,
                );
                let particles_interaction = (particles_distance / 2.) - 1.;

                if particles_interaction > 0. {
                    density[particles_cursor as usize] = density[particles_cursor as usize]
                        + particles_interaction * particles_interaction;
                }
            }
        }

        println!("jsp");

        for particles_cursor in 0..total_of_particles {
            y_force[particles_cursor as usize] = GRAVITY;
            x_force[particles_cursor as usize] = 0.;
            for particles_cursor2 in 0..total_of_particles {
                x_particle_distance =
                    x_pos[particles_cursor as usize] - x_pos[particles_cursor2 as usize];
                y_particle_distance =
                    y_pos[particles_cursor as usize] - y_pos[particles_cursor2 as usize];
                let particles_distance = sqrt(
                    (x_particle_distance * x_particle_distance
                        + y_particle_distance * y_particle_distance) as f32,
                );
                let particles_interaction = (particles_distance / 2.) - 1.;

                if particles_interaction > 0. {
                    x_force[particles_cursor as usize] += particles_interaction
                        * (x_particle_distance as f32
                            * (3. - density[particles_cursor as usize])
                            * PRESSURE
                            + x_velocity[particles_cursor as usize] * VISCOSITY
                            - x_velocity[particles_cursor2 as usize] * VISCOSITY)
                        / (density[particles_cursor as usize]);
                    y_force[particles_cursor as usize] += particles_interaction
                        * (y_particle_distance as f32
                            * (3. - density[particles_cursor as usize])
                            * PRESSURE
                            + y_velocity[particles_cursor as usize] * VISCOSITY
                            - y_velocity[particles_cursor2 as usize] * VISCOSITY)
                        / (density[particles_cursor as usize]);
                }
            }
        }

        println!("Loul");

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                SCREENBUFFER[i][j] = 0 as u8;
            }
        }

        for particles_cursor in 0..total_of_particles {
            if wall_flag[particles_cursor as usize] == 0 {
                if sqrt(
                    x_force[particles_cursor as usize] * x_force[particles_cursor as usize]
                        + y_force[particles_cursor as usize] * y_force[particles_cursor as usize],
                ) < 4.2
                {
                    x_velocity[particles_cursor as usize] +=
                        x_force[particles_cursor as usize] / 10.;
                    y_velocity[particles_cursor as usize] +=
                        y_force[particles_cursor as usize] / 10.;
                } else {
                    x_velocity[particles_cursor as usize] +=
                        x_force[particles_cursor as usize] / 11.;
                    y_velocity[particles_cursor as usize] +=
                        y_force[particles_cursor as usize] / 11.;
                }
                x_pos[particles_cursor as usize] += x_velocity[particles_cursor as usize] as isize;
                y_pos[particles_cursor as usize] += y_velocity[particles_cursor as usize] as isize;
            }

            let x = x_pos[particles_cursor as usize] as usize;
            let y = (y_pos[particles_cursor as usize] / 2) as usize;

            if y < HEIGHT - 1 && x < WIDTH - 1 {
                SCREENBUFFER[x][y] |= 8;
                SCREENBUFFER[x + 1][y] |= 4;
                SCREENBUFFER[x][y + 1] |= 2;
                SCREENBUFFER[x + 1][y + 1] |= 1;
            }
        }

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                let index = SCREENBUFFER[i][j];
                print!("{}", CHARS[index as usize]);
            }
            println!();
        }
    }
}
