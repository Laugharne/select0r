extern crate num_cpus;
extern crate crypto;

use std::env;
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


#[derive(Debug)]
enum Output {
	TSV,
	CSV,
	JSON,
	XML,
}


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
	signature  : String,
	part_name  : String,
	part_args  : String,
	difficulty : u32,
	nn_threads : u32,
	digit_max  : u32,
	decrease   : bool,
	results    : Vec<Signature>,
	max_results: u32,
	output     : Output,
}


#[derive(Debug)]
struct Signature {
	signature: String,
	selector : u32,
	nn_zero  : u32,
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
	let zero_counter: usize = (&selector_u8_vec[..4]).iter().filter(|&&x| x == 0).count();
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
		selector : selector_u32,
		nn_zero  : zero_counter,
	})

}


fn main_process(mut g: Globals) {

	let hasher: crypto::sha3::Sha3 = crypto::sha3::Sha3::keccak256();
	let mut optimal: u32  = u32::MAX;

	(1..=g.digit_max).for_each( |digit| {
		let max: IteratedValue = 1 << (BASE_BITS*digit);
		//println!("{} : {}", digit, max);

		print!("Pass #{} : ", digit);
		//(0..max).step_by(g.nn_threads).for_each( |value| {
		(0..max).for_each( |value| {
			match compute(&g, hasher, digit, value) {
				None => {},
				Some(s) => {
					if g.decrease == true {
						if s.selector < optimal {
							optimal = s.selector;
							//println!("  [{:>08X}]\t{}", s.selector, s.signature);
							print!("■");
							g.results.push( s);
						}
					} else {
						//println!("  [{:>08X}]\t{}", s.selector, s.signature);
						print!("■");
						g.results.push( s);
					}

					if g.results.len() >= g.max_results as usize {
						write_file(&g);
						process::exit(0);
					}
				}
			};
			//if value > 20 {break;}	// just for debug purpose !
		});
		println!("");
	});
	println!("");

}


fn write_file(mut g: &Globals) {
	let file_name: String = format!("{}--zero={}-max={}-decr={}-cpu={}.{:?}", g.signature, g.difficulty, g.max_results, g.decrease, g.nn_threads, g.output);
	let mut csv_file: Result<File, std::io::Error> = File::create(file_name);
	match csv_file {
		Ok(ref mut f) => {
			let format: &str = match g.output {
				Output::JSON => "{\n",
				Output::XML  => "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<select0r>\n",
				_ => "",
			};
			let _ = f.write(format.as_bytes());

			for line in &g.results {
				let line_csv: String = match g.output {
					Output::TSV  =>	format!("{:>08x}\t{}\n", line.selector, line.signature),
					Output::CSV  =>	format!("\"{:>08x}\",\"{}\"\n", line.selector, line.signature),
					Output::JSON =>	format!("\t{{ \"selector\":\"{:>08x}\", \"signature\":\"{}\" }},\n", line.selector, line.signature),
					Output::XML  =>	format!("\t<result>\n\t\t<selector>{:>08x}</selector>\n\t\t<signature>{}</signature>\n\t</result>\n", line.selector, line.signature),
				};
				let _ = f.write(line_csv.as_bytes());
			}

			let format: &str = match g.output {
				Output::JSON => "}\n",
				Output::XML  => "</select0r>\n",
				_ => "",
			};
			let _ = f.write(format.as_bytes());

		},
		Err(_e) => panic!(),
	}

}

fn print_help() {
	// eprintln
	// equivalent to println!() except the output goes to
	// standard err (stderr) instead of standard output (stdio)
	eprintln!(
		"\n{} - Selector Optimizer, find better EVM function name to optimize Gas cost",
		"Select0r".green().bold()
	);
	eprintln!("Usage : select0r s <function_signature string> z <number_of_zeros> r <max_results> d <decrement boolean> t <nbr_threads> o <format_ouput>");
	eprintln!("");
	eprintln!("Example 1 : select0r s \"functionName(uint256)\"  z 2  r 5  d true  t 2  o tsv");
	eprintln!("Example 2 : select0r s \"functionName2(uint)\"  z 2  r 7  d false  t 2  o json");
}


fn init_app() -> Globals {

	println!("");
	println!("  .--.--.               ,--,                          ___        ,----..             ");
	println!(" /  /    '.           ,--.'|                        ,--.'|_     /   /   \\            ");
	println!("|  :  /`. /           |  | :                        |  | :,'   /   .     :   __  ,-. ");
	println!(";  |  |--`            :  : '                        :  : ' :  .   /   ;.  \\,' ,'/ /| ");
	println!("|  :  ;_       ,---.  |  ' |      ,---.     ,---. .;__,'  /  .   ;   /  ` ;'  | |' | ");
	println!(" \\  \\    `.   /     \\ '  | |     /     \\   /     \\|  |   |   ;   |  ; \\ ; ||  |   ,' ");
	println!("  `----.   \\ /    /  ||  | :    /    /  | /    / ':__,'| :   |   :  | ; | ''  :  /   ");
	println!("  __ \\  \\  |.    ' / |'  : |__ .    ' / |.    ' /   '  : |__ .   |  ' ' ' :|  | '    ");
	println!(" /  /`--'  /'   ;   /||  | '.'|'   ;   /|'   ; :__  |  | '.'|'   ;  \\; /  |;  : |    ");
	println!("'--'.     / '   |  / |;  :    ;'   |  / |'   | '.'| ;  :    ; \\   \\  ',  / |  , ;    ");
	println!("  `--'---'  |   :    ||  ,   / |   :    ||   :    : |  ,   /   ;   :    /   ---'     ");
	println!("             \\   \\  /  ---`-'   \\   \\  /  \\   \\  /   ---`-'     \\   \\ .'             ");
	println!("              `----'             `----'    `----'                `---`               ");
	println!("");


	// manage cli parameters
	let args: Vec<String> = env::args().skip(1).collect();
	let mut arg_signature  : String = "".to_string();
	let mut arg_difficulty : u32    = 2;
	let mut arg_max_results: u32    = 4;
	let mut arg_decrease   : bool   = false;
	let mut arg_threads    : u32    = 2;
	let mut arg_output     : Output = Output::TSV;

	if (args.len() & 1) != 0 {
		print_help();
		eprintln!(
			"{} wrong number of parameters given. Got {}\n",
			"Error".red().bold(),
			args.len()
		);

		process::exit(1);
	}

	enum NextIs{
		NOTHING,
		SIGNATURE,
		ZERO,
		RESULTS,
		DECREASE,
		THREADS,
		OUTPUT,
	}

	let mut next: NextIs = NextIs::NOTHING;

	for arg in &args {
		//println!("- {}", arg);
		match next {
			NextIs::SIGNATURE => { arg_signature   = arg.to_string();},
			NextIs::ZERO      => { arg_difficulty  = arg.parse::<u32>().unwrap().clamp(1,3);},
			NextIs::RESULTS   => { arg_max_results = arg.parse::<u32>().unwrap().clamp(2,10);},
			NextIs::DECREASE  => { arg_decrease    = match arg.as_str() {"1"|"true"|"TRUE"=>true, "0"|"false"|"FALSE"=>false, _=>panic!("Invalid decrease value")};},
			NextIs::THREADS   => { arg_threads     = arg.parse::<u32>().unwrap().clamp( 1, num_cpus::get() as u32);},
			NextIs::OUTPUT    => {arg_output = match arg.as_str() {
									"tsv" |"TSV"|"" => Output::TSV,
									"csv" |"CSV"    => Output::CSV,
									"json"|"JSON"   => Output::JSON,
									"xml" |"XML"    => Output::XML,
									_               => panic!("Invalid output value")
								};},
			_                 => {},
		}
		next = NextIs::NOTHING;

		match arg.as_str() {
			"s"|"S" => { next = NextIs::SIGNATURE;},
			"z"|"Z" => { next = NextIs::ZERO;},
			"r"|"R" => { next = NextIs::RESULTS;},
			"d"|"D" => { next = NextIs::DECREASE;},
			"t"|"T" => { next = NextIs::THREADS;},
			"o"|"O" => { next = NextIs::OUTPUT;},
			_       => { next = NextIs::NOTHING;/* TODO */},
		}

	}

	if arg_signature.len() <= 0 {
		print_help();
		panic!("No signature !?");
	}

	println!("");
	println!("- Signature\t`{}`",        arg_signature);
	println!("- Difficulty\t{} zero(s)", arg_difficulty);
	println!("- Max results\t{}",        arg_max_results);
	println!("- Decrease\t{}",           arg_decrease);
	println!("- Nbr threads\t{} CPU(s)", arg_threads);
	println!("- Output\t{:?} file",      arg_output);
	println!("");

	let parenthesis: usize = arg_signature.find("(").unwrap();
	let part_n: &str       = &arg_signature[..parenthesis];
	let part_a: &str       = &arg_signature[parenthesis..];
	let digit: u32         = (f64::log(IteratedValue::MAX as f64, BASE_NN as f64) as u32) + 1;

	Globals {
		signature  : arg_signature.clone(),
		part_name  : part_n.to_owned(),
		part_args  : part_a.to_owned(),
		difficulty : arg_difficulty,
		nn_threads : arg_threads,
		digit_max  : digit,
		decrease   : arg_decrease,
		results    : vec![],
		max_results: arg_max_results,
		output     : arg_output,
	}

}


fn main() {
	let mut g: Globals = init_app();
	//println!("{:?}", g);
	main_process( g);
	process::exit(0);

}


//	time cargo run s "aaaa(uint)"  z 2  d false  t 2
//	time cargo run s "aaaa(uint)"  z 1  d true  t 3
//	time cargo run s "deposit(uint256)"  z 2  d true  t 3 r 8 o tsv
//					s signature
//					z (nbr zero)
//					r nbr results needed
//					d decrease values
//					t nbr of threads (clamp by app)