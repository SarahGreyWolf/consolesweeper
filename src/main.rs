#![feature(exclusive_range_pattern)]

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
        Self {
            state,
            value,
        }
    }
}
impl Into<u8> for Cell {
    fn into(self) -> u8 {
        let mut out = 0x00;
        match self.state {
            State::Hidden => out += 0x00,
            State::Revealed => out += 0x10,
            State::Marked => out += 0x20,
            State::Unknown => out += 0x30,
        }
        out += self.value;
        out
    }
}

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

struct GameState {
    field: [u8; WIDTH*HEIGHT],
    score: u64,
    running: bool,
    losses: u64,
    wins: u64
}

impl GameState {
    fn init() -> Self {
        Self {
            field: [0x00; WIDTH*HEIGHT],
            score: 0,
            running: true,
            losses: 0,
            wins: 0,
        }
    }

    fn run(&mut self) {
        let mut in_accepted = false;
        loop {
            if !self.running {
                break;
            }
            in_accepted = false;
            draw(&self.field);
            println!("Please enter where you would like to target in the format 'x y command', or exit");
            println!("Where command is reveal or mark");
            while !in_accepted {
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).expect("Failed to get stdin");
                if buffer == "exit\r\n" {
                    println!("Exiting...");
                    self.running = false;
                    break;
                }
                in_accepted = self.execute(&buffer);
                if !in_accepted {
                    println!("The command you entered was incorrect");
                    println!("Please enter the command as 'x y command' where command is reveal or mark");
                }
            }

        }
    }

    fn execute(&mut self, command: &str) -> bool {
        let split: Vec<String> = command.split_ascii_whitespace().map(|s| s.to_string()).collect();
        if split.len() == 3 {
            let command = split[2].to_lowercase();
            let x = split[0].parse::<u16>().expect("Failed to parse string to u16");
            let y = split[1].parse::<u16>().expect("Failed to parse string to u16");
            if x as usize > WIDTH || y as usize > HEIGHT {
                return false;
            }
            let mut cell = Cell::from(self.field[x as usize + y as usize * WIDTH]);
            match command.as_str() {
                "reveal" => {
                    if cell.state == State::Hidden || cell.state == State::Marked {
                        cell.state = State::Revealed;
                    }
                },
                "mark" => {
                    cell.state = State::Marked;
                },
                _ => {
                    return false;
                }
            }
            self.field[x as usize + y as usize * WIDTH] = cell.into();
            return true;
        } else {
            false
        }
    }
}

fn main() {
    let mut game = GameState::init();
    game.run();
//    field[42] = 0x19;
//    draw(&field);
}

fn draw(field: &[u8; WIDTH*HEIGHT]) {
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
                State::Revealed => {
                    match cell.value {
                        1..8 => print!("{:#}", cell.value),
                        9 => print!("*"),
                        _ => print!(" "),
                    }
                },
                State::Marked => print!("?"),
                State::Unknown => panic!("Unknown Cell State of {:#} at {:#}:{:#}", (&field[x+y*WIDTH] >> 4), x, y),
            }
        }
        print!("]\n");
    }
}

fn reveal(field: &mut [u8; WIDTH*HEIGHT]) {
    
}
