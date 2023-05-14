#![feature(exclusive_range_pattern)]

use rand::prelude::*;
use std::io::{self, Read};

#[derive(Debug, PartialEq, Eq)]
enum State {
    Hidden,
    Revealed,
    Marked,
    Unknown,
}

struct Cell {
    state: State,
    value: u8,
}

impl From<u8> for Cell {
    fn from(cell: u8) -> Self {
        let state = {
            match (cell >> 4) {
                0 => State::Hidden,
                1 => State::Revealed,
                2 => State::Marked,
                _ => State::Unknown,
            }
        };
        let value = cell & 0x0F;
        Self { state, value }
    }
}

impl From<Cell> for u8 {
    fn from(cell: Cell) -> Self {
        let mut out = 0x00;
        match cell.state {
            State::Hidden => out += 0x00,
            State::Revealed => out += 0x10,
            State::Marked => out += 0x20,
            State::Unknown => out += 0x30,
        }
        out += cell.value;
        out
    }
}

const WIDTH: usize = 30;
const HEIGHT: usize = 30;

struct GameState {
    field: [u8; WIDTH * HEIGHT],
    mine_positions: Vec<[u16; 2]>,
    score: u64,
    running: bool,
    losses: u64,
    wins: u64,
}

impl GameState {
    fn init() -> Self {
        let mut field: [u8; WIDTH * HEIGHT] = [0x00; WIDTH * HEIGHT];
        let mut mine_positions: Vec<[u16; 2]> = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..WIDTH {
            let x: u16 = rng.gen_range(0, WIDTH as u16);
            let y: u16 = rng.gen_range(0, HEIGHT as u16);
            mine_positions.push([x, y]);
            field[x as usize + y as usize * WIDTH] = 0x09;
            println!("Mine at {x}:{y}");
            for y_pos in -1 as i16..=1 as i16 {
                for x_pos in -1 as i16..=1 as i16 {
                    if x_pos == 0 && y_pos == 0 {
                        continue;
                    }
                    if x as i16 + x_pos > 1 && ((x as i16 + x_pos) as usize) < WIDTH {
                        if y as i16 + y_pos > 1 && ((y as i16 + y_pos) as usize) < HEIGHT {
                            if field[((x as i16 + x_pos) as usize)
                                + ((y as i16 + y_pos) as usize) * WIDTH]
                                == 0x09
                            {
                                continue;
                            }
                            field[((x as i16 + x_pos) as usize)
                                + ((y as i16 + y_pos) as usize) * WIDTH] += 1;
                        }
                    }
                }
            }
        }

        Self {
            field,
            mine_positions,
            score: 0,
            running: true,
            losses: 0,
            wins: 0,
        }
    }

    fn run(&mut self) {
        let mut coord: [u16; 2] = [0; 2];
        loop {
            let mut in_accepted = false;
            draw(&self.field);
            if !self.running {
                break;
            }
            println!(
                "Please enter where you would like to target in the format 'x y command', or exit"
            );
            println!("Where command is reveal or mark");
            while !in_accepted {
                let mut buffer = String::new();
                std::io::stdin()
                    .read_line(&mut buffer)
                    .expect("Failed to get stdin");
                if buffer == "exit\r\n" {
                    println!("Exiting...");
                    self.running = false;
                    break;
                }
                let (n_in_accepted, n_coord) = self.execute(&buffer);
                in_accepted = n_in_accepted;
                coord = n_coord.unwrap();
                if !in_accepted {
                    println!("The command you entered was incorrect");
                    println!(
                        "Please enter the command as 'x y command' where command is reveal or mark"
                    );
                }
            }
            let cell = Cell::from(self.field[coord[0] as usize + coord[1] as usize * WIDTH]);
            if cell.value == 9 && cell.state != State::Marked {
                self.running = false;
                reveal_bombs(&mut self.field);
            }
        }
        println!("Game Over!");
    }

    fn execute(&mut self, command: &str) -> (bool, Option<[u16; 2]>) {
        let split: Vec<String> = command
            .split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect();
        if split.len() == 3 {
            let command = split[2].to_lowercase();
            let x = split[0]
                .parse::<u16>()
                .expect("Failed to parse string to u16");
            let y = split[1]
                .parse::<u16>()
                .expect("Failed to parse string to u16");
            if x as usize > WIDTH || y as usize > HEIGHT {
                return (false, Some([x, y]));
            }
            let mut cell = Cell::from(self.field[x as usize + y as usize * WIDTH]);
            match command.as_str() {
                "reveal" => {
                    if cell.state == State::Hidden || cell.state == State::Marked {
                        cell.state = State::Revealed;
                        reveal([x, y], &mut self.field);
                    }
                }
                "mark" => {
                    cell.state = State::Marked;
                }
                _ => {
                    return (false, Some([x, y]));
                }
            }
            self.field[x as usize + y as usize * WIDTH] = cell.into();
            return (true, Some([x, y]));
        } else {
            (false, None)
        }
    }
}

fn main() {
    let mut game = GameState::init();
    game.run();
    //    draw(&field);
}

fn draw(field: &[u8; WIDTH * HEIGHT]) {
    print!("  ");
    for x in 0..WIDTH {
        print!("{:#}", x);
    }
    print!("\n");
    for y in 0..HEIGHT {
        print!("{:#}[", y);
        for x in 0..WIDTH {
            let cell = Cell::from(field[x + y * WIDTH]);
            match cell.state {
                State::Hidden => print!("\u{2588}"),
                State::Revealed => match cell.value {
                    1..8 => print!("{:#}", cell.value),
                    9 => print!("*"),
                    _ => print!(" "),
                },
                State::Marked => print!("?"),
                State::Unknown => panic!(
                    "Unknown Cell State of {:#} at {:#}:{:#}",
                    (&field[x + y * WIDTH] >> 4),
                    x,
                    y
                ),
            }
        }
        print!("]\n");
    }
}

fn reveal(origin: [u16; 2], field: &mut [u8; WIDTH * HEIGHT]) {
    let cell_origin = Cell::from(field[origin[0] as usize + origin[1] as usize * WIDTH]);
    if origin[0] > 0
        && (origin[0] as usize) < WIDTH
        && origin[1] > 0
        && (origin[1] as usize) < HEIGHT
        && (origin[0] as usize) + (origin[1] as usize) * WIDTH < WIDTH * HEIGHT
    {
        //println!("{:#}:{:#}", origin[0], origin[1]);
        for c in 0..9 {
            let mut pos: [u16; 2] = [0; 2];
            if c < 3 {
                pos = [origin[0] + c - 1, origin[1] - 1];
            } else if 2 < c && c < 6 {
                for x in 0..2 {
                    pos = [origin[0] + x - 1, origin[1]];
                }
            } else {
                for x in 0..2 {
                    pos = [origin[0] + x - 1, origin[1] + 1];
                }
            }
            if (pos[0] as usize) + (pos[1] as usize) * WIDTH < WIDTH * HEIGHT {
                let mut n_cell = Cell::from(field[pos[0] as usize + pos[1] as usize * WIDTH]);
                if pos != origin && n_cell.state != State::Revealed && n_cell.value != 9 {
                    if n_cell.value < 9 && n_cell.value == 0 {
                        n_cell.state = State::Revealed;
                        field[pos[0] as usize + pos[1] as usize * WIDTH] = n_cell.into();
                        reveal(pos, field);
                    }
                }
            }
        }
    }
}

fn reveal_bombs(field: &mut [u8; WIDTH * HEIGHT]) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut cell = Cell::from(field[x + y * WIDTH]);
            if cell.value == 9 {
                cell.state = State::Revealed;
                field[x + y * WIDTH] = cell.into();
            }
        }
    }
}
