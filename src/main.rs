extern crate num_cpus;
extern crate crypto;

use std::env;
use std::error::Error;
use std::process;
use std::f64;
use crypto::digest::Digest;
use self::crypto::sha3::Sha3;
use text_colorizer::*;

use std::fs::File;
use std::io::prelude::*;


type  IteratedValue           = u32;	//u64;
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
	results     : Vec<Signature>,
	max_results : u32,
}


#[derive(Debug)]
struct Signature {
	signature : String,
	selector  : u32,
	nn_zero   : u32,
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



fn compute(g: &Globals, mut hasher: Sha3, digit: u32, value: IteratedValue) -> Option<Signature> {
	let value64: String   = base64_to_string(digit, value).unwrap();
	let signature: String = format!("{}_{}{}",g.part_name ,value64, g.part_args );

	hasher.reset();
	hasher.input_str(&signature);
	let mut selector_u8_vec: [u8; 32] = [0; 32];
	hasher.result(&mut selector_u8_vec);
/*
	let zero_counter: usize   = (&selector_u8_vec[..4]).iter().filter(|&&x| x == 0).count();
	let mut leading_zero: u32 = 0;

	let selector_u32: u32   = ((selector_u8_vec[0] as u32) << 24)
							+ ((selector_u8_vec[1] as u32) << 16)
							+ ((selector_u8_vec[2] as u32) << 8)
							+   selector_u8_vec[3] as u32;
*/
	let mut zero_counter: u32 = 0;
	let mut selector_u32: u32 = 0;
	for i in 0..4 {
		if selector_u8_vec[i] == 0 {
			zero_counter += 1;
		}

		selector_u32 = (selector_u32<<8) + (selector_u8_vec[i] as u32);
	}

	if selector_u32 == 0 {return None;}
	if zero_counter < g.difficulty {return None;}

	//println!("{:>8x}\t{}\t{:?}", selector_u32, signature, &selector_u8_vec[..4]);

	Some( Signature {
		signature: signature,
		selector:  selector_u32,
		nn_zero:   zero_counter,
	})

}


fn main_process(mut g: Globals) {

	let hasher: crypto::sha3::Sha3 = crypto::sha3::Sha3::keccak256();
	let mut optimal: u32  = u32::MAX;

	(1..=g.digit_max).for_each( |digit| {
		let max: IteratedValue = 1 << (BASE_BITS*digit);
		//println!("{} : {}", digit, max);

		println!("Brute force, pass #{}", digit);
		//(0..max).step_by(g.nn_threads).for_each( |value| {
		(0..max).for_each( |value| {
			match compute(&g, hasher, digit, value) {
				None => {},
				Some(s) => {
					if (g.leading_zero == true) {
						if (s.selector < optimal) {
							optimal = s.selector;
							println!("  [{:>08x}]\t{}", s.selector, s.signature);
							g.results.push( s);
						}
					} else {
						println!("  [{:>08x}]\t{}", s.selector, s.signature);
						g.results.push( s);
					}
					if g.results.len() >= g.max_results as usize {
						write_tsv(&g);
						process::exit(0);
					}
				}
			};
			//if value > 20 {break;}	// just for debug purpose !
		});
		println!("");
	});
}


fn write_tsv(mut g: &Globals) {
	let file_name: String = format!("{}--zero={}-max={}.tsv", g.signature, g.difficulty, g.max_results);
	let mut csv_file: Result<File, std::io::Error> = File::create(file_name);
	match csv_file {
		Ok(ref mut f) => {
			for line in &g.results {
				let line_csv: String = format!("{:>08x}\t{}\n", line.selector, line.signature);
				let _ = f.write(line_csv.as_bytes());
			}
		},
		Err(_e) => panic!(),
	}

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
	if args.len() != 4 {
		print_help();
		eprintln!(
			"{} wrong number of Globals give. Expected 4, got {}\n",
			"Error".red().bold(),
			args.len()
		);

		process::exit(1);
	}

	let parenthesis: usize = args[0].find("(").unwrap();
	let part_n: &str = &args[0][..parenthesis];
	let part_a: &str = &args[0][parenthesis..];
	let _digit: u32  = f64::log(IteratedValue::MAX as f64, BASE_NN as f64) as u32;

	Globals {
		signature   : args[0].clone(),
		part_name   : part_n.to_owned(),
		part_args   : part_a.to_owned(),
		difficulty  : args[1].parse::<u32>().unwrap(),
		nn_threads  : num_cpus::get() as u32,
		digit_max   : _digit+1,
		leading_zero: match &*args[3] {"true"=>true, "false"=>false, _=>panic!("invalid leading zero value")},
		results     : vec![],
		max_results	: args[2].parse::<u32>().unwrap(),
	}

}


fn main() {
	let mut g: Globals = init_app();
	//println!("{:?}", g);
	main_process( g);
	process::exit(0);

}


//	time cargo run "aaaa(uint)" 2 10 true
//	time cargo run "aaaa(uint)" 1 10 true

// TODO later !
//	time cargo run s "aaaa(uint)" z 1 l true t 3
//					s signature
//					z (nbr zero)
//					l leading zero
//					t nbr of threads (clamp by app)