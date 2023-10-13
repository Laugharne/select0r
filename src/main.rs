use std::env;
use std::process;
use std::f64;
use text_colorizer::*;


const SEPARATOR: u8 = '_' as u8;

type  IterateValue           = u32; // u64 !?
const BASE_NN: IterateValue  = 64;
const BASE_MAX: IterateValue = BASE_NN-1;
const BASE_BITS: u32         = BASE_MAX.count_ones();

//const DIGIT_MAX: u32 = 2;	// 2 = test / 5 = release


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
	digit_max   : u32,
	leading_zero: bool,
}



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


fn init_app() -> Globals {
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

	let digit: u32 = f64::log(IterateValue::MAX as f64, BASE_NN as f64) as u32;

	Globals {
		signature   : args[0].clone(),
		part_name   : part_n.to_owned(),
		part_args   : part_a.to_owned(),
		difficulty  : args[1].parse::<u32>().unwrap(),
		nn_threads  : 0,
		digit_max   : 2,//digit+1,
		leading_zero: match &*args[2] {"true"=>true, "false"=>false, _=>panic!("invalid leading zero value")},
	}

}


fn compute() {

}


fn base64_to_suffix( digit: u32, value: IterateValue) -> Result<String, std::string::FromUtf8Error> {
	const ALPHABET: &[u8; BASE_NN as usize] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";

	let mut value: IterateValue       = value;
	let mut str_u8: [u8; 12] = [0; 12];

	let mut da: usize = str_u8.len()-1;
	(0..digit).for_each(|_| {
		str_u8[da]   = ALPHABET[(value & BASE_MAX) as usize];
		value      >>= BASE_BITS;
		da -= 1;
	});

	str_u8[da] = SEPARATOR;

	//println!("{:?}", str_u8);
	let string: Result<String, std::string::FromUtf8Error> = String::from_utf8(str_u8[da..].to_vec());
	match string {
		Ok(str) => Ok(str),
		Err(e) => Err(e),
	}
}


fn main_process(g: &Globals) {

	(1..=g.digit_max).for_each(|digit| {
		let max: IterateValue = 1 << (BASE_BITS*digit);
		//println!("{} : {}", digit, max);
		for value in 0..max {

			match base64_to_suffix(digit, value) {
				Ok(chaine) => {
					println!("{}", chaine);
				}
				Err(e) => {
					println!("Erreur de conversion : {:?}", e);
				}
			}
			//if value > 10 {break;}

		}
	});
}


fn main() {
	let g: Globals = init_app();

	//println!("{:?}", g);

	main_process( &g);

	process::exit(0);

}

/*


use std::f64;

fn main() {
	let valeur: f64 = 64.0; // Exemple de valeur numérique

	// Calcul du logarithme en base 64
	let log_base_64 = f64::log(valeur, 64.0);

	println!("Le logarithme en base 64 de {} est : {}",
		valeur, (log_base_64 as u32)+1);
}

*/

//	time cargo run "aaaa(uint)" 2 true
