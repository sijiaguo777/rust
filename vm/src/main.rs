
extern crate pretty_env_logger;
extern crate log;
fn main() -> Result<(), interpreter::Error> {
    env_logger::init();
    pretty_env_logger::init(); // set logger

    // Take a filename as argument on the command line
    let filename = std::env::args().nth(1).unwrap();

    // Read content to buffer
    let buffer = std::fs::read(filename).unwrap();

    // Create a machine with this memory content and run it
    let mut machine = interpreter::Machine::new(&buffer).unwrap();
    machine.run()
}
