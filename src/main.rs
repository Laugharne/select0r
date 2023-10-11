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

const BASE_NN: u32    = 64;
const BASE_MAX: u32   = BASE_NN-1;
const BASE_BITS: u32  = BASE_MAX.count_ones();


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
	//println!("{:?}", args);
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


fn base64_to_string( digit: u32, value: u32) -> Result<String, std::string::FromUtf8Error> {
	const ALPHABET: &[u8; BASE_NN as usize] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";

	let mut value: u32       = value;
	let mut str_u8: [u8; 10] = [0; 10];

	let mut da: usize = str_u8.len()-1;
	for _d in 0..digit {
		str_u8[da]   = ALPHABET[(value & BASE_MAX) as usize];
		value      >>= BASE_BITS;
		da -= 1;
	}

	//println!("{:?}", str_u8);
	let string: Result<String, std::string::FromUtf8Error> = String::from_utf8(str_u8.to_vec());
	match string {
		Ok(str) => Ok(str),
		Err(e) => Err(e),
	}
}



fn main_process(g: &Globals) {

	//for digit in 1..=5 {
	for digit in 1..=2 {
		let max: u32 = 1 << (BASE_BITS*digit);
		//println!("{} : {}", digit, max);
		for value in 0..max {
			let b64 = base64_to_string(digit, value);
			match b64 {
				Ok(chaine) => {
					println!("{}", chaine);
				}
				Err(e) => {
					println!("Erreur de conversion : {:?}", e);
				}
			}
		}
	}
}


fn main() {
	let g: Globals = parse_args();

	//println!("{:?}", g);

	main_process( &g);

	process::exit(0);

}

//  "aaaa(uint)" 2 true