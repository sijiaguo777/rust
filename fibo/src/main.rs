use clap::Parser;

fn fibo(n: u32) -> Option<u32> {
    if n == 0 {
        return Some(0);
    } else if n == 1 {
        return Some(1);
    }

    let a: u32 = fibo(n - 1)?; // ? is a shortcut for match
    let b: u32 = fibo(n - 2)?;

    a.checked_add(b)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "The maximal number to print the fibo value of")]
    value: u32,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long, value_name = "NUMBER")]
    min: u32,
}

fn main() {
    // fibo recursive
    let args = Args::parse();
    let min = args.min;
    for i in min..=args.value {
        let result = fibo(i);
        match result {
            Some(x) => println!("fibo({}) = {}", i, x),
            None => println!("Overflow! current u32 can't handle fibo({})", i),
        }
    }
}
