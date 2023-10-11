use std::env;
use std::process;
use text_colorizer::*;


//	#[derive(Debug)]
// this is just going to allow us to use
// the Standard output a little better
// to kind of format
//
//	#[allow(dead_code)]
// allow us to suppress warnings bind to dead code
#[derive(Debug)]
#[allow(dead_code)]
struct Globals {
	signature   : String,
	part_name   : String,
	part_args   : String,
	difficulty  : u32,
	nn_threads  : u32,
	leading_zero: bool,
}

const SEPARATOR: &str = "_";



fn print_help() {
	// eprintln
	// equivalent to println!() except the output goes to
	// standard err (stderr) instead of standard output (stdio)
	eprintln!(
		"\n{} - find better EVM function name to optimize Gas cost",
		"Selector Optimizer".green()
	);
	eprintln!("Usage :   <function_signature string> <difficulty number> <leading_zero boolean>");
	eprintln!("Example : \"functionName(uint)\" 2 true");
}


fn parse_args() -> Globals {
	let args: Vec<String> = env::args().skip(1).collect();
	println!("{:?}", args);
	if args.len() != 3 {
		print_help();
		eprintln!(
			"{} wrong number of Globals give. Expected 3, got {}\n",
			"Error".red().bold(),
			args.len()
		);

		process::exit(1);
	}

	let parenthesis: usize = args[0].find("(").unwrap();
	let part_n: &str = &args[0][..parenthesis];
	let part_a: &str = &args[0][parenthesis..];
	
	Globals{
		signature   : args[0].clone(),
		part_name   : part_n.to_owned(),
		part_args   : part_a.to_owned(),
		difficulty  : args[1].parse::<u32>().unwrap(),
		nn_threads  : 0,
		leading_zero: match &*args[2] {"true"=>true,"false"=>false,_=>panic!("invalid leading zero value")},
	}

}


fn compute() {

}


fn main_process(g: &Globals) {

	
	for digit in 1..=5 {
		let max: u32 = 1 << (6*digit);
		//println!("{} : {}", digit, max);
		// TODO
	}
}


fn main() {
	let g: Globals = parse_args();

	println!("{:?}", g);

	main_process( &g);

	process::exit(0);

}

//  "aaaa(uint)" 2 true