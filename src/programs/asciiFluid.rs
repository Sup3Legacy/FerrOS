use core::num;
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
    let mut xSandboxAeraScan = 0;
    let mut ySandboxAeraScan = 0;
    let mut xPos = [0 as isize; 2 * HEIGHT * WIDTH];
    let mut yPos = [0 as isize; 2 * HEIGHT * WIDTH];
    let mut xForce = [0.; 2 * HEIGHT * WIDTH];
    let mut yForce = [0.; 2 * HEIGHT * WIDTH];
    let mut xVelocity = [0.; 2 * HEIGHT * WIDTH];
    let mut yVelocity = [0.; 2 * HEIGHT * WIDTH];
    let mut density = [0.; 2 * HEIGHT * WIDTH];
    let mut wallFlag = [0 as isize; 2 * HEIGHT * WIDTH];
    let mut totalOfParticles = 0 as isize;
    let mut SCREENBUFFER = [[0 as u8; HEIGHT]; WIDTH];

    let mut particlesCounter = 0;

    for i in 0..WIDTH {
        for j in 0..HEIGHT {
            if i == 0 || j == 0 || i >= WIDTH - 2 || j >= HEIGHT - 3 {
                wallFlag[particlesCounter] = 1;
                wallFlag[particlesCounter + 1] = 1;
                totalOfParticles += 2;
                xPos[particlesCounter] = i as isize;
                yPos[particlesCounter] = 2 * j as isize;
                xPos[particlesCounter + 1] = i as isize;
                yPos[particlesCounter + 1] = (2 * j + 1) as isize;
                particlesCounter += 2;
            } else if i <= WIDTH / 2 && j >= 10 {
                wallFlag[particlesCounter] = 0;
                wallFlag[particlesCounter + 1] = 0;
                totalOfParticles += 2;
                particlesCounter += 2;
                xPos[particlesCounter] = i as isize;
                yPos[particlesCounter] = 2 * j as isize;
                xPos[particlesCounter + 1] = i as isize;
                yPos[particlesCounter + 1] = (2 * j + 1) as isize;
            }
        }
    }

    let mut xParticleDistance;
    let mut yParticleDistance;
    let a = (3 as f32) * 9.;
    println!("{}", sqrt(100.));
    println!("Initialized!");
    loop {
        println!("Loop beginning");
        for particlesCursor in 0..totalOfParticles {
            println!("{} {}", particlesCursor, 256 as f32 * 1.1);
            let valeur = wallFlag[particlesCursor as usize];
            println!("mdr");
            //density[particlesCursor as usize] = valeur + 0.0001;
            
            for particlesCursor2 in 0..totalOfParticles {
                println!("{}", particlesCursor2);
                xParticleDistance = xPos[particlesCursor as usize] - xPos[particlesCursor2 as usize];
                yParticleDistance = yPos[particlesCursor as usize] - yPos[particlesCursor2 as usize];
                let particlesDistance = sqrt(
                    (xParticleDistance * xParticleDistance + yParticleDistance * yParticleDistance)
                        as f32,
                );
                let particlesInteraction = (particlesDistance / 2.) - 1.;

                if particlesInteraction > 0. {
                    density[particlesCursor as usize] =
                        density[particlesCursor as usize] + particlesInteraction * particlesInteraction;
                }
            }
        }

        println!("jsp");

        for particlesCursor in 0..totalOfParticles {
            yForce[particlesCursor as usize] = GRAVITY;
            xForce[particlesCursor as usize] = 0.;
            for particlesCursor2 in 0..totalOfParticles {
                xParticleDistance = xPos[particlesCursor as usize] - xPos[particlesCursor2 as usize];
                yParticleDistance = yPos[particlesCursor as usize] - yPos[particlesCursor2 as usize];
                let particlesDistance = sqrt(
                    (xParticleDistance * xParticleDistance + yParticleDistance * yParticleDistance)
                        as f32,
                );
                let particlesInteraction = (particlesDistance / 2.) - 1.;

                if particlesInteraction > 0. {
                    xForce[particlesCursor as usize] += particlesInteraction
                        * (xParticleDistance as f32 * (3. - density[particlesCursor as usize]) * PRESSURE
                            + xVelocity[particlesCursor as usize] * VISCOSITY
                            - xVelocity[particlesCursor2 as usize] * VISCOSITY)
                        / (density[particlesCursor as usize]);
                    yForce[particlesCursor as usize] += particlesInteraction
                        * (yParticleDistance as f32 * (3. - density[particlesCursor as usize]) * PRESSURE
                            + yVelocity[particlesCursor as usize] * VISCOSITY
                            - yVelocity[particlesCursor2 as usize] * VISCOSITY)
                        / (density[particlesCursor as usize]);
                }
            }
        }

        println!("Loul");

        for i in 0..WIDTH {
            for j in 0..HEIGHT {
                SCREENBUFFER[i][j] = 0 as u8;
            }
        }

        for particlesCursor in 0..totalOfParticles {
            if wallFlag[particlesCursor as usize] == 0 {
                if sqrt(
                    xForce[particlesCursor as usize] * xForce[particlesCursor as usize]
                        + yForce[particlesCursor as usize] * yForce[particlesCursor as usize],
                ) < 4.2
                {
                    xVelocity[particlesCursor as usize] += xForce[particlesCursor as usize] / 10.;
                    yVelocity[particlesCursor as usize] += yForce[particlesCursor as usize] / 10.;
                } else {
                    xVelocity[particlesCursor as usize] += xForce[particlesCursor as usize] / 11.;
                    yVelocity[particlesCursor as usize] += yForce[particlesCursor as usize] / 11.;
                }
                xPos[particlesCursor as usize] += xVelocity[particlesCursor as usize] as isize;
                yPos[particlesCursor as usize] += yVelocity[particlesCursor as usize] as isize;
            }

            let x = xPos[particlesCursor as usize] as usize;
            let y = (yPos[particlesCursor as usize] / 2) as usize;

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
