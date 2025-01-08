use emulator::Processor;
use crate::emulator::Display;

mod emulator;

fn main() {
    // let mut cpu = Processor::new();
    // let rom = include_bytes!("random_number_test.ch8").to_vec();
    // cpu.load_rom(rom);
    // cpu.run();

    let mut display = Display::new();
    display.draw(1, 3, (&[0x7C, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x7C], 7));
    display.draw(3, 5, (&[0x7C, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x7C], 7));
    for y in 0..32 {
        for x in 0..64 {
            print!("{}", if display.get_pixel(x, y) { "██" } else { "░░" })
        }
        println!()
    }
}
