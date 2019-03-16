use rand::prelude::*;
use minifb::{Key, WindowOptions, Window};
use std::{thread, time};

use std::time::{UNIX_EPOCH, SystemTime};

use noise::{Perlin, Seedable};
use noise::utils::*;
use noise::NoiseFn;

const SIZE: usize = 600;
const NUMBER_OF_ORGANISMS: usize = 100;
const REPRO_LIMIT : i32 = 2;

pub struct Organism {
    pub name: String,
    pub carnivore_fitness: f32,
    pub herbivore_fitness: f32,
    pub color: u32,
    pub can_eat: [bool; NUMBER_OF_ORGANISMS]
}

impl Organism {
    pub fn change_can_eat(&mut self) {
        if rand::thread_rng().gen::<f32>() > 0.95 {        
            for i in 0..NUMBER_OF_ORGANISMS {
                self.can_eat[i] = rand::thread_rng().gen::<f32>() < 0.65;
            }
        }
    }
}

pub struct Simulation {
    pub organisms: Vec<Organism>,
    pub populations: [i32; NUMBER_OF_ORGANISMS],
    pub food: [i32; NUMBER_OF_ORGANISMS],
    pub map: [[i32; SIZE]; SIZE],
    pub food_map: [[f32; SIZE]; SIZE],
    pub lava_map: [[bool; SIZE]; SIZE],
    pub random_source: ThreadRng,
    look_functions: Vec<fn(&mut Simulation, usize, usize, i32, &mut bool) -> ()>
}

impl Simulation {

    pub fn init(&mut self) {
       for y in 0..SIZE {
            for x in 0..SIZE {
                self.map[y][x] = self.random_source.gen_range(0, NUMBER_OF_ORGANISMS as i32);
                self.populations[self.map[y][x] as usize] += 1;
            }
        }
    }

    pub fn decay_lava(&mut self) {
        for y in 0..SIZE {
            for x in 0..SIZE {
                if self.lava_map[y][x] == true {
                    self.lava_map[y][x] = self.random_source.gen::<f32>() < 0.95f32;
                }
            }
        }
    }

    pub fn run_simulation_once(&mut self, mutate: bool) { 
        self.try_eat();
        self.starve();
        self.reproduce();
        if mutate {
            self.mutate();
            self.decay_lava();    
        }
    }

    pub fn mutate(&mut self) {
        let mut highest_index = 0;
        let mut lowest_index = 0;

        for i in 0..NUMBER_OF_ORGANISMS {
            if self.populations[i] > self.populations[highest_index] {
                highest_index = i;
            }
            if self.populations[i] < self.populations[lowest_index] {
                lowest_index = i;
            }
        }

        let mut best_organism = self.organisms.get_mut(highest_index).unwrap();
        best_organism.carnivore_fitness = best_organism.carnivore_fitness / 2.0;

        let mut worst_organism = self.organisms.get_mut(lowest_index).unwrap();
        worst_organism.carnivore_fitness = worst_organism.carnivore_fitness * 2.0;


        for i in 0..NUMBER_OF_ORGANISMS {
            self.organisms[i].change_can_eat();
        }

    }

    pub fn look_up(&mut self, x: usize, y: usize, me: i32, eaten: &mut bool) {
        let mut y_value = y as i32 - 1;
        if y_value < 0 {
            y_value = SIZE as i32 - 1;
        }
        let them = self.map[y_value as usize][x];
        if self.can_eat(me, them) && *eaten == false {
            self.map[y_value as usize][x] = -1;
            self.food[me as usize] += 1;
            self.populations[them as usize] -= 1;
            *eaten = true;
        }
    }

    pub fn look_left(&mut self, x: usize, y: usize, me: i32, eaten: &mut bool) {
        let mut x_value = x as i32 - 1;
        if x_value < 0 {
            x_value = SIZE as i32- 1;
        }

        let them = self.map[y][x_value as usize];
        if self.can_eat(me, them) == true && *eaten == false {
            self.map[y][x_value as usize] = -1;
            self.food[me as usize] += 1;
            self.populations[them as usize] -= 1;
            *eaten = true;
        }
    }

    pub fn look_right(&mut self, x: usize, y: usize, me: i32, eaten: &mut bool) {
        
        let them = self.map[y][(x + 1) % SIZE];
        if self.can_eat(me, them) == true && *eaten == false {
            self.map[y][(x + 1)%SIZE] = -1;
            self.food[me as usize] += 1;
            self.populations[them as usize] -= 1;
            *eaten = true;
        }
    }
    
    pub fn look_down(&mut self, x: usize, y: usize, me: i32, eaten: &mut bool) {
        let them = self.map[(y + 1)%SIZE][x];
        if self.can_eat(me, them) && *eaten == false {
            self.map[(y + 1)%SIZE][x] = -1;
            self.food[me as usize] += 1;
            self.populations[them as usize] -= 1;
            *eaten = true;
        }
    }

    pub fn try_eat(&mut self) {
        let mut range_y : Vec<usize> = (0..SIZE).collect();
        self.random_source.shuffle(&mut range_y);
        for y in range_y {
            let mut range_x : Vec<usize> = (0..SIZE).collect();
            self.random_source.shuffle(&mut range_x);

            for x in range_x {
                let me = self.map[y][x];
                if me == -1 {
                    continue;
                }

                let mut eaten = false;
                /*
                self.random_source.shuffle(&mut self.look_functions);

                for look_func in self.look_functions.clone() {
                    look_func(self, x, y, me, &mut eaten);
                }
                */
                
                self.look_up(x, y, me, &mut eaten);
                self.look_right(x, y, me, &mut eaten);
                self.look_down(x, y, me, &mut eaten);
                self.look_left(x, y, me, &mut eaten);
                
                /*
                let mut x_value = x as i32 - 1;
                if x_value < 0 {
                    x_value = SIZE as i32- 1;
                }

                them = self.map[y][x_value as usize];
                if self.can_eat(me, them) == true {
                    self.map[y][x_value as usize] = -1;
                    self.food[me as usize] += 1;
                    self.populations[them as usize] -= 1;
                    eaten = true;
                }

                let them = self.map[y][(x + 1) % SIZE];
                if self.can_eat(me, them) == true && eaten == false {
                    self.map[y][(x + 1)%SIZE] = -1;
                    self.food[me as usize] += 1;
                    self.populations[them as usize] -= 1;
                    eaten = true;
                }

                let mut y_value = y as i32 - 1;
                if y_value < 0 {
                    y_value = SIZE as i32 - 1;
                }
                let them = self.map[y_value as usize][x];
                if self.can_eat(me, them) && eaten == false {
                    self.map[y_value as usize][x] = -1;
                    self.food[me as usize] += 1;
                    self.populations[them as usize] -= 1;
                    eaten = true;
                }

                let them = self.map[(y + 1)%SIZE][x];
                if self.can_eat(me, them) && eaten == false {
                    self.map[(y + 1)%SIZE][x] = -1;
                    self.food[me as usize] += 1;
                    self.populations[them as usize] -= 1;
                    eaten = true;
                }

                */
                let food_there = self.random_source.gen::<f32>() > self.food_map[y][x];
                let i_eat = self.random_source.gen::<f32>() < self.organisms[me as usize].herbivore_fitness;

                if food_there && i_eat && eaten == false {
                    self.food[me as usize] += 1;
                }
            }
        }
    }

    pub fn starve(&mut self) {
        let mut range_y : Vec<usize> = (0..SIZE).collect();
        self.random_source.shuffle(&mut range_y);

        for y in range_y {
            let mut range_x : Vec<usize> = (0..SIZE).collect();
            self.random_source.shuffle(&mut range_x);

            for x in range_x {
                let me = self.map[y][x];
                if me == -1 {
                    continue;
                }

                if self.lava_map[y][x] == true {
                    self.populations[me as usize] -= 1;
                    self.map[y][x] = -1;
                }
                else if self.die(me as usize) {
                    self.populations[me as usize] -= 1;
                    self.map[y][x] = -1;
                }
                else {
                    self.food[me as usize] -= 1;
                    if self.food[me as usize] < 0 {
                        self.food[me as usize] = 0;
                    }
                }
            }
        }
    }

    pub fn reproduce(&mut self) {

        let mut is_dead = vec![];

        for i in 0..self.populations.len() {
            if self.populations[i] == 0 {
                is_dead.push(i);
            }
        }

        

        let mut range_y : Vec<usize> = (0..SIZE).collect();
        self.random_source.shuffle(&mut range_y);
        for y in range_y {
            let mut range_x : Vec<usize> = (0..SIZE).collect();
            self.random_source.shuffle(&mut range_x);

            for x in range_x {
                if self.map[y][x] != -1 {
                    continue;
                }

                let mut number_around = [0i32; NUMBER_OF_ORGANISMS + 1];
                let x_plus = (x + 1) % SIZE;
                let y_plus = (y + 1) % SIZE;

                let mut x_min = x;
                if x == 0 {
                    x_min = SIZE - 1;
                }
                else {
                    x_min = x - 1;
                }
 
                let mut y_min = y;
                if y == 0 {
                    y_min = SIZE - 1;
                }
                else {
                    y_min = y - 1;
                }

                number_around[(self.map[y_min][x_min] + 1) as usize ] += 1;
                number_around[(self.map[y][x_min] + 1) as usize] += 1;
                number_around[(self.map[y_plus][x_min] + 1) as usize] += 1;
                number_around[(self.map[y_min][x] + 1) as usize] += 1;

                number_around[(self.map[y_plus][x] + 1) as usize] += 1;
                number_around[(self.map[y_min][x_plus] + 1) as usize] += 1;
                number_around[(self.map[y][x_plus] + 1) as usize] += 1;
                number_around[(self.map[y_plus][x_plus] + 1) as usize] += 1;

                let mut max_species = 0;
                let mut max = 0;
                for i in 1..NUMBER_OF_ORGANISMS {
                    if number_around[i] > max {
                        max = number_around[i];
                        max_species = i;
                    }
                }

                if max >= REPRO_LIMIT {
                    max_species -= 1;
                    self.map[y][x] = max_species as i32;
                    self.populations[max_species] += 1;
                }
            }
        }
    }

    pub fn print_map(&self) {
        for row in self.map.iter() {
            for col in row.iter() {
                if *col != -1 {
                    print!("{} ", col);
                }
                else {
                    print!("E ");
                }
            }
            print!("\n");
        }
        print!("\n");
    }

    pub fn die(&mut self, me: usize) -> bool {
        return self.random_source.gen::<f32>() * self.populations[me] as f32 > self.food[me] as f32;
    }

    pub fn can_eat(&mut self, me: i32, them: i32) -> bool {
        return them != (-1) && self.organisms[me as usize].can_eat[them as usize] == true && me != them && self.random_source.gen::<f32>() < (self.organisms[me as usize].carnivore_fitness - self.organisms[them as usize].carnivore_fitness);
    }
}

fn main() {
    
    let mut perlin = Perlin::new();
    let val = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
    perlin = perlin.set_seed(val);

    println!("{}", val);    
        
    let mut food_map = [[0.0f32;SIZE]; SIZE];
    for y in 0..SIZE {
        for x in 0..SIZE {

            food_map[y][x] = rand::thread_rng().gen::<f32>();
        }
    }

    for y in 200..250 {
        for x in 200..250 {
            food_map[y][x] = 0.0f32;
        }
    }

    let mut lava_map = [[false; SIZE];SIZE];

    for y in 0..SIZE {
        for x in 0..SIZE {
        let value = perlin.get([y as f64 / 100.0, x  as f64 / 100.0]);
            if value < -0.7 {
                lava_map[y][x] = true;
            }
        }
    }

    let mut simulation: Simulation = 
                        Simulation{organisms:vec![], 
                                   populations: [0i32; NUMBER_OF_ORGANISMS],
                                   food: [0i32; NUMBER_OF_ORGANISMS],
                                   map:[[0i32;SIZE]; SIZE],
                                   random_source: rand::thread_rng(),
                                   look_functions: vec![Simulation::look_down, Simulation::look_up, Simulation::look_left, Simulation::look_right],
                                   food_map,
                                   lava_map
                                   };

    let mut buffer : Vec<u32> = vec![0; SIZE * SIZE];
    let mut window = Window::new("LIFE", SIZE, SIZE, WindowOptions::default()).unwrap_or_else(|e| {
        panic!("{}", e);
    });
    let slice_of_color = std::u32::MAX / NUMBER_OF_ORGANISMS as u32;
    for i in 0..NUMBER_OF_ORGANISMS {
        let mut can_eat = [false; NUMBER_OF_ORGANISMS];
        for j in 0..NUMBER_OF_ORGANISMS {
            if j != i {
                can_eat[j] = rand::thread_rng().gen::<f32>() < 0.65;
            }
            else {
                can_eat[j] = false;
            }
        } 
        let org = Organism{
            name: i.to_string(),
            carnivore_fitness: rand::thread_rng().gen::<f32>(),
            herbivore_fitness: rand::thread_rng().gen::<f32>(),
            color: (slice_of_color * i as u32).rotate_left(rand::thread_rng().gen::<u32>()),
            can_eat: can_eat
        };
        simulation.organisms.push(org);
    }

    
    
    simulation.init();
    let mut frame : i64 = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut count = 0;
        simulation.run_simulation_once(frame % 100 == 0);

        for i in buffer.iter_mut() {
            let organism = simulation.map[count / SIZE][count % SIZE];
            if simulation.lava_map[count / SIZE][count % SIZE] == false {
                if organism != -1 {
                    *i = simulation.organisms[organism as usize].color;
                }
                else {
                    let red   = 125.0;
                    let green = 255.0;
                    let blue  = 125.0;
                    let final_color = ((red as u32) << 16 | (green as u32) << 8 | (blue as u32)).into();
                    *i = final_color;
                }
            }
            else {
                let red   = 255.0 * 1.0;
                let green = 102.0;
                let blue  = 255.0 * 0.0;

                let final_color = ((red as u32) << 16 | (green as u32) << 8 | (blue as u32)).into();
                *i = final_color;
            }


            count += 1;
        }
        window.update_with_buffer(&buffer).unwrap();
        frame += 1;
    }
}
