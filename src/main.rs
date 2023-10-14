extern crate num_cpus;
extern crate crypto;

use std::env;
use std::process;
use std::f64;
use crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use text_colorizer::*;


type  IteratedValue           = u64;
const BASE_NN: IteratedValue  = 64;
const BASE_MAX: IteratedValue = BASE_NN-1;
const BASE_BITS: u32          = BASE_MAX.count_ones();



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
	//hasher      : crypto::sha3::Sha3, 
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

	let _digit: u32 = f64::log(IteratedValue::MAX as f64, BASE_NN as f64) as u32;

	Globals {
		signature   : args[0].clone(),
		part_name   : part_n.to_owned(),
		part_args   : part_a.to_owned(),
		difficulty  : args[1].parse::<u32>().unwrap(),
		nn_threads  : num_cpus::get() as u32,
		digit_max   : 2,//digit+1,
		leading_zero: match &*args[2] {"true"=>true, "false"=>false, _=>panic!("invalid leading zero value")},
	}

}


fn base64_to_string( digit: u32, value: IteratedValue) -> Result<String, std::string::FromUtf8Error> {
	const ALPHABET: &[u8; BASE_NN as usize] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";

	let mut value: IteratedValue = value;
	let mut str_u8: [u8; 12]     = [0; 12];
	let mut da: usize            = str_u8.len()-1;

	(0..digit).for_each( |_| {
		str_u8[da]   = ALPHABET[(value & BASE_MAX) as usize];
		value      >>= BASE_BITS;
		da -= 1;
	});

	da += 1;

	//println!("{:?}", str_u8);
	let string: Result<String, std::string::FromUtf8Error> = String::from_utf8(str_u8[da..].to_vec());
	match string {
		Ok(str) => Ok(str),
		Err(e) => Err(e),
	}
}


fn compute(g: &Globals, mut hasher: Sha3, digit: u32, value: IteratedValue) {
	let value64: String   = base64_to_string(digit, value).unwrap();
	let signature: String = format!("{}_{}{}",g.part_name ,value64, g.part_args );

	hasher.reset();
	hasher.input_str(&signature);
	let mut selector_vu8: [u8; 32] = [0; 32];
	hasher.result(&mut selector_vu8);

	let mut zero_counter = (&selector_vu8[..4]).iter().filter(|&&x| x == 0).count();

	let selector_u32: u32   = ((selector_vu8[0] as u32) << 24)
							+ ((selector_vu8[1] as u32) << 16)
							+ ((selector_vu8[2] as u32) << 8)
							+   selector_vu8[3] as u32;

	println!("{:>8x}\t{}\t{}", selector_u32, signature, zero_counter);
	//println!("{:>8x}\t{}\t{:?}", selector_u32, signature, &selector_vu8[..4]);
}


fn main_process(g: &Globals) {

	let hasher: crypto::sha3::Sha3 = crypto::sha3::Sha3::keccak256();

	(1..=g.digit_max).for_each( |digit| {
		let max: IteratedValue = 1 << (BASE_BITS*digit);
		//println!("{} : {}", digit, max);

		//(0..max).step_by(g.nn_threads).for_each( |value| {
		//(0..max).for_each( |value| {
		for value in 0..max {	// still use `for in` for the moment to use `break` instruction (at the end of the loop)
			compute(g, hasher, digit, value);
			//if value > 20 {break;}	// just for debug purpose !
		//});
		}
	});
}


fn main() {
	let g: Globals = init_app();

	println!("{:?}", g);

	/*
	let mut hasher: crypto::sha3::Sha3 = crypto::sha3::Sha3::keccak256();
	let signature: &str = "deposit278591A(uint)";
	hasher.input_str(&signature);
	let hash_result: String = hasher.result_str();
	assert_eq!(&hash_result[..8], "00000070");
	println!("{}\t{}", &hash_result[..8], signature);

	hasher.reset();
	hasher.input_str(&signature);
	let mut out: [u8; 32]     = [0; 32];
	hasher.result(&mut out);
	println!("{:?}", &out[..4]);
	*/

	main_process( &g);

	process::exit(0);

}


//	time cargo run "aaaa(uint)" 2 true
