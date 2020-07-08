use std::env;
use std::error;
use std::fs;
use std::io::Read;

pub mod disasm;
pub mod emulator;

fn main() -> Result<(), Box<dyn error::Error>> {
    let game_path: String = env::args().nth(1).expect("No filename found in arguments");
    let mut file_contents: Vec<u8> = Vec::new(); 

    fs::File::open(game_path)?.read_to_end(&mut file_contents)?;

    let asm = disasm::disasm(&file_contents)?;
    println!("{}", asm);

    let mut runner = emulator::State::new(&mut file_contents);
    runner.start();

    Ok(())
}
