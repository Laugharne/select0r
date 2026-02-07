extern crate num_cpus;
extern crate crypto;

use std::io::prelude::*;
use std::f64;
use std::fs::File;
use std::env;
use text_colorizer::*;

use crypto::digest::Digest;
use self::crypto::sha3::Sha3;

use std::process;
//use std::thread;
use crossbeam::thread;
use std::sync::Mutex;



#[macro_use]
extern crate lazy_static;
lazy_static! {
	static ref SHARED_RESULTS: Mutex<Vec<SignatureResult>> = Mutex::new(
		Vec::new()
	);
}


type  IteratedValue           = u32;	// u64;

const BASE_NN: IteratedValue  = 64;
const BASE_MAX: IteratedValue = BASE_NN-1;
const BASE_BITS: u32          = BASE_MAX.count_ones();

const LOW: &str   = "▦";
const FOUND: &str = "■";
const STAR: &str  = "★";


#[derive(Clone)]
#[derive(Debug)]
enum Output {
	TSV,
	CSV,
	JSON,
	XML,
	RON,
}


//	#[derive(Debug)]
// this is just going to allow us to use
// the Standard output a little better
// to kind of format
//
//	#[allow(dead_code)]
// allow us to suppress warnings bind to dead code
#[derive(Debug)]
#[derive(Clone)]
#[allow(dead_code)]
struct Globals {
	signature  : String,
	part_name  : String,
	part_args  : String,
	difficulty : u32,
	nn_threads : usize,
	digit_max  : u32,
	leading0   : bool,
	results    : Vec<SignatureResult>,
	max_results: usize,
	output     : Output,
}


struct SelectorResult {
	selector    : u32,
	zero_counter: u32,
}


/// The `SignatureResult` struct represents the result of a signature operation, containing a signature
/// string, a selector value, and a leading zero count.
///
/// Properties:
///
/// * `signature`: A string that represents a valid Solidity signature.
/// * `selector`: The `selector` property is of type `u32`, which stands for unsigned 32-bit integer. It
/// is used to store a numeric value that represents a selector.
/// * `leading_zero`: The `leading_zero` property is of type `u32`, which stands for unsigned 32-bit
/// integer. It represents the number of leading zeros in the binary representation of the `signature`
/// property.
#[derive(Clone)]
#[derive(Debug)]
struct SignatureResult {
	signature   : String,
	selector    : u32,
	leading_zero: u32,
	nbr_of_zero : u32,
}


/// The function `base64_to_string` converts a given digit and value into a string using a specific
/// alphabet.
///
/// Arguments:
///
/// * `digit`: The `digit` parameter represents the number of digits in the base64 value that you want
/// to convert to a string.
/// * `value`: The `value` parameter in the `base64_to_string` function is of type `IteratedValue`. It
/// represents the value that needs to be converted from base64 to a string.
///
/// Returns:
///
/// The function `base64_to_string` returns a `String` as we know that it's a valid UTF-8 string.
fn base64_to_string(digit: u32, mut value: IteratedValue) -> String {
    const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";

    let mut buffer: Vec<u8> = vec![0u8; digit as usize];

    for i in (0..digit as usize).rev() {
        buffer[i] =   ALPHABET[(value & BASE_MAX) as usize];
        value     >>= BASE_BITS;
    }

    // Direct conversion (we know that ALPHABET is UTF-8 valid)
    unsafe { String::from_utf8_unchecked(buffer) }
}


/// The function takes a signature as input, hashes it using SHA3, and converts the resulting hash into
/// a selector by counting the number of leading zeros and converting the first 4 bytes into a u32
/// value.
///
/// Arguments:
///
/// * `signature`: The `signature` parameter is a string that represents a function signature. It is
/// used to generate a selector, which is a unique identifier for the function.
/// * `hasher`: The `hasher` parameter is an instance of the `Sha3` struct, which is used to compute the
/// SHA-3 hash of the input signature. It is passed as a mutable reference to the function so that it
/// can be reset and reused for multiple computations.
///
/// Returns:
///
/// The function `signature_to_selector` returns a `SelectorResult` struct.
fn signature_to_selector(signature: &str, mut hasher: Sha3) -> SelectorResult {

	hasher.reset();
	hasher.input_str(signature);
	let mut selector_u8_vec: [u8; 32] = [0; 32];
	hasher.result(&mut selector_u8_vec);

	let (zero_counter, selector_u32) = selector_u8_vec
		.iter()
		.take(4)
		.enumerate()
		.fold((0, 0), |(zero_counter, selector_u32), (_i, &vu8)| {(
			if vu8 == 0 { zero_counter + 1 } else { zero_counter },
			(selector_u32 << 8) + (vu8 as u32),
		)});

	SelectorResult {
		selector    : selector_u32,
		zero_counter: zero_counter,
	}
}


/// The function counts the number of leading zeros in a 32-bit unsigned integer.
///
/// Arguments:
///
/// * `selector_u32`: The parameter `selector_u32` is an unsigned 32-bit integer.
///
/// Returns:
///
/// If none of the conditions in the if statements are true, then the function will return 4.
fn count_leading_zeros(selector_u32: u32) -> u32 {
	if (selector_u32 & 0xFF000000) != 0 { return 0;}
	if (selector_u32 & 0x00FF0000) != 0 { return 1;}
	if (selector_u32 & 0x0000FF00) != 0 { return 2;}
	if (selector_u32 & 0x000000FF) != 0 { return 3;}
	4
}


/// The function takes in some parameters, computes a signature result based on those parameters, and
/// returns it as an option.
///
/// Arguments:
///
/// * `g`: A reference to a struct called `Globals` which contains global variables and settings for the
/// computation.
/// * `digit`: The `digit` parameter is of type `u32` and represents the number of base 64 digits used in the
/// computation.
/// * `value`: The `value` parameter is of type `IteratedValue`. It represents some value that has been
/// iterated over.
/// * `hasher`: The `hasher` parameter is of type `Sha3`, which is a hash function. It is used to
/// compute the hash value of the `signature` string.
///
/// Returns:
///
/// The function `compute` returns an `Option<SignatureResult>`.
fn compute(g: &Globals, digit: u32, value: IteratedValue, hasher: Sha3) -> Option<SignatureResult> {
	let value64: String     = base64_to_string(digit, value);
	let signature: String   = format!("{}_{}{}",g.part_name ,value64, g.part_args );
	let s2s: SelectorResult = signature_to_selector(&signature, hasher);
	let selector_u32: u32   = s2s.selector;
	let zero_counter: u32   = s2s.zero_counter;

	//if selector_u32 == 0 {return None;}
	if zero_counter < g.difficulty {return None;}

	//println!("{:>8x}\t{}\t{:?}", selector_u32, signature, &selector_u8_vec[..4]);
	let leading_zero = count_leading_zeros(selector_u32);

	Some( SignatureResult {
		signature   : signature,
		selector    : selector_u32,
		leading_zero: leading_zero,
		nbr_of_zero : zero_counter,
	})

}


/// The function `thread` takes in some parameters and performs computations using a hashing algorithm,
/// updating shared variables and printing progress along the way.
///
/// Arguments:
///
/// * `g`: The parameter `g` is of type `Globals` and represents a struct that contains global variables
/// and settings for the program.
/// * `idx`: The `idx` parameter represents the starting index for the iteration. It is used to
/// determine the range of values that the `for_each` loop will iterate over.
/// * `digit`: The `digit` parameter is of type `u32` and represents the number of base 64 digits.
/// * `max`: The `max` parameter represents the maximum value for the iteration. It is of type
/// `IteratedValue`.
fn thread(g: Globals, idx: IteratedValue, digit: u32, max: IteratedValue) {
	let hasher: crypto::sha3::Sha3 = crypto::sha3::Sha3::keccak256();
	let mut optimal:u32       = u32::MAX;  // TODO !
	let mut nn_results: usize = 1;         // TODO !
	{
		let shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().expect("Mutex panic ! ");
		if let Some(last_signature) = shared.last() {
			optimal    = last_signature.selector;
			nn_results = shared.len();
		}
	}

	(idx..max).step_by(g.nn_threads).for_each( |value| {
		match compute(&g, digit, value, hasher) {
			None => {},
			Some(s) => {

				if g.leading0 {
					if s.selector < optimal {
						optimal = s.selector;
						{
							let mut shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().expect("Mutex panic ! ");
							if let Some(last_signature) = shared.last() {

								nn_results = shared.len();

								let shared_optimal: u32 = last_signature.selector;

								match shared_optimal.cmp(&optimal) {
									std::cmp::Ordering::Less => {
										optimal = shared_optimal;
									}
									std::cmp::Ordering::Greater => {
										print!("{}", in_progress(s.leading_zero));
										shared.push( SignatureResult {
											signature   : s.signature,
											selector    : optimal,
											leading_zero: s.leading_zero,
											nbr_of_zero : s.nbr_of_zero,
										});
										nn_results += 1;
									}
									_ => {}
								}


							}// if let Some(last_signature)
						}// SHARED_RESULTS.lock()
					}
				} else {
					//println!("  [{:>08X}]\t{}", s.selector, s.signature);
					print!("{}", in_progress(s.leading_zero));
					let mut shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().expect("Mutex panic ! ");
					shared.push( SignatureResult{
						signature   : s.signature,
						selector    : s.selector,
						leading_zero: s.leading_zero,
						nbr_of_zero : s.nbr_of_zero,
					});
					nn_results = shared.len();
				}

				if (optimal == 0) || (nn_results >= g.max_results) {
					write_file(&g, "Goal reached !");
					process::exit(0);
					//return;
				}

			}// Some()
		};// match compute()
	});// step_by(g.nn_threads).for_each(value)

}


/// The function `in_progress` takes an input `nn_zeros` and returns a colored string based on its
/// value.
///
/// Arguments:
///
/// * `nn_zeros`: The parameter `nn_zeros` is of type `u32`, which stands for unsigned 32-bit integer.
/// It represents the number of zeros to indicate the progress status.
///
/// Returns:
///
/// The function `in_progress` returns a `ColoredString`.
fn in_progress(nn_zeros: u32) -> ColoredString {
	match nn_zeros {
		1 => FOUND.to_string().red(),
		2 => FOUND.to_string().yellow().bold(),
		3 => FOUND.to_string().green(),
		4 => STAR.to_string().green(),
		_ => LOW.to_string().bright_red(),
	}
}


/// Launches multiple threads to perform operations based on the provided Global configuration.
///
/// This function pushes a `SignatureResult` into the shared results,
/// launches multiple threads for each pass based on number of base64 digit
/// and performs operations based on the Global configuration and specified digits.
///
/// Arguments:
///
/// * `g`: The parameter `g` is of type `&Globals`, which means it is a reference to an object of type
/// `Globals`.
fn threads_launcher(g: &Globals) -> &Globals {
	{
		let mut shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().unwrap();

		let hasher: crypto::sha3::Sha3 = crypto::sha3::Sha3::keccak256();
		let s2s: SelectorResult        = signature_to_selector(&g.signature, hasher);

		shared.push(SignatureResult {
			signature   : g.signature.clone(),
			selector    : s2s.selector,
			leading_zero: count_leading_zeros(s2s.selector),
			nbr_of_zero : s2s.zero_counter,
		});
	}

	(1..=g.digit_max).for_each( |digit| {
		print!("Pass #{} ", digit);
		let max: IteratedValue = 1 << (BASE_BITS*digit);

		let _ = thread::scope(|scope| {
			(0..g.nn_threads).for_each(|thread_idx| {
				scope.spawn(move |_| {
					thread( g.clone(), thread_idx as IteratedValue, digit, max);
				});
			});
		});

		println!();

	});// for_each( digit)
	println!("\n");
	g

}


/// The function `write_file` takes in a `Globals` struct and a message string, creates a file name
/// based on the struct's properties, and writes the message and the struct's data to a file in the
/// specified format.
///
/// Arguments:
///
/// * `g`: A reference to a struct called `Globals` which contains various configuration parameters for
/// the file writing process.
/// * `message`: A message to be display in standard output, before writing to the file.
fn write_file(g: & Globals, message: & str) {
	let file_name: String = format!("select0r-{}--zero={}-max={}-lead={}-cpu={}.{:?}",
						g.signature, g.difficulty, g.max_results, g.leading0, g.nn_threads, g.output);

	println!("\n\n{}", message.green());
	println!("\nOutput : {}\n", file_name.cyan());
	let mut csv_file: Result<File, std::io::Error> = File::create(file_name);

	match csv_file {
		Ok(ref mut f) => {
			let format: &str = match g.output {
				Output::TSV  => "SELECTOR\tNBR_OF_ZERO\tLEADING_ZERO\tSIGNATURE\n",
				Output::CSV  => "\"SELECTOR\",\"NBR_OF_ZERO\",\"LEADING_ZERO\",\"SIGNATURE\"\n",
				Output::JSON => "{\"select0r\":[\n",
				Output::XML  => "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<select0r>\n",
				Output::RON  => "Select0r( results: [\n",
			};
			let _ = f.write(format.as_bytes());

			let shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().expect("Mutex panic ! ");
			for (line_idx, 	line) in shared.iter().enumerate() {
				let comma_or_not = if line_idx==0 {" "}else{","};

				let line_csv: String = match g.output {
					Output::TSV  =>	format!("{:>08x}\t{}\t{}\t{}\n", line.selector, line.nbr_of_zero, line.leading_zero, line.signature),
					Output::CSV  =>	format!("\"{:>08x}\",{},{},\"{}\"\n", line.selector, line.nbr_of_zero, line.leading_zero, line.signature),
					Output::JSON =>	format!("\t{}{{ \"selector\":\"{:>08x}\", \"nbr_of_zero\":\"{}\", \"leading_zero\":\"{}\", \"signature\":\"{}\" }}\n"
						,comma_or_not, line.selector, line.nbr_of_zero, line.leading_zero, line.signature),
					Output::XML  =>	format!("\t<result>\n\t\t<selector>{:>08x}</selector>\n\t\t<nbr_of_zero>{}</nbr_of_zero>\n\t\t<leading_zero>{}</leading_zero>\n\t\t<signature>{}</signature>\n\t</result>\n", line.selector, line.nbr_of_zero, line.leading_zero, line.signature),
					Output::RON  => format!("\t{}(selector: \"{:>08x}\", nbr_of_zero: {}, leading_zero: {}, signature: \"{}\")\n"
						,comma_or_not, line.selector, line.nbr_of_zero, line.leading_zero, line.signature),
				};
				let _ = f.write(line_csv.as_bytes());
			}

			let format: &str = match g.output {
				Output::JSON => "]}\n",
				Output::XML  => "</select0r>\n",
				Output::RON  => "],)\n",
				_            => "",
			};
			let _ = f.write(format.as_bytes());

		},
		Err(_e) => panic!(),
	}

}


/// Displays the command-line interface (CLI) help information for the Select0r application.
///
/// This function provides guidance on the usage of the Select0r tool, including examples
/// of how to use it with various command-line arguments.
fn cli_help() {
	// eprintln
	// equivalent to println!() except the output goes to
	// standard err (stderr) instead of standard output (stdio)
	eprintln!(
		"\n{} - Selector Optimizer, find better function name to optimize gas cost",
		"Select0r".green().bold()
	);
	eprintln!("Usage : select0r s <function_signature string> z <number_of_zeros> r <max_results> d <decrement boolean> t <nbr_threads> o <format_ouput>");
	eprintln!();
	eprintln!("Example 1 : select0r s \"functionName(uint256)\"  z 2  r 5  l true  t 2  o tsv");
	eprintln!("Example 2 : select0r s \"functionName2(uint)\"  z 2  r 7  l false  t 2  o json");
	eprintln!();
}


/// The `init_app` function initializes the application by parsing command line arguments and setting up
/// global variables.
///
/// Returns:
///
/// The function `init_app()` returns a `Globals` struct.
fn init_app() -> Globals {
// TODO intercept ctrl-c to stop processing and write results on output file !

	println!();
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
	println!();


	// manage cli parameters
	let args: Vec<String> = env::args().skip(1).collect();
	let mut arg_signature  : String = "".to_string();
	let mut arg_difficulty : u32    = 2;
	let mut arg_max_results: u32    = 4;
	let mut arg_leading0   : bool   = false;
	let mut arg_threads    : usize  = 2;
	let mut arg_output     : Output = Output::TSV;

	if (args.len() & 1) != 0 {
		cli_help();
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
		LEADING0,
		THREADS,
		OUTPUT,
	}

	let mut _next: NextIs = NextIs::NOTHING;

	for arg in &args {
		//println!("- {}", arg);
		match _next {
			NextIs::SIGNATURE => { arg_signature   = arg.to_string();},
			NextIs::ZERO      => { arg_difficulty  = arg.parse::<u32>().expect("Invalid `z`parameter ! ").clamp(1,3);},
			NextIs::RESULTS   => { arg_max_results = arg.parse::<u32>().expect("Invalid `r` parameter ! ").clamp(2,20);},
			NextIs::LEADING0  => { arg_leading0    = match arg.as_str() {"1"|"true"|"TRUE"=>true, "0"|"false"|"FALSE"=>false, _=>panic!("Invalid `l` parameter ! ")};},
			NextIs::THREADS   => { arg_threads     = arg.parse::<usize>().expect("Invalid `t` parameter ! ").clamp( 1, num_cpus::get());},
			NextIs::OUTPUT    => { arg_output = match arg.as_str() {
									"tsv" |"TSV"|"" => Output::TSV,
									"csv" |"CSV"    => Output::CSV,
									"json"|"JSON"   => Output::JSON,
									"xml" |"XML"    => Output::XML,
									"ron" |"RON"    => Output::RON,
									_               => panic!("Invalid `o` parameter ! ")
								};},
			_                 => {},
		}
		_next = NextIs::NOTHING;

		match arg.as_str() {
			"s"|"S" => { _next = NextIs::SIGNATURE;},
			"z"|"Z" => { _next = NextIs::ZERO;},
			"r"|"R" => { _next = NextIs::RESULTS;},
			"l"|"L" => { _next = NextIs::LEADING0;},
			"t"|"T" => { _next = NextIs::THREADS;},
			"o"|"O" => { _next = NextIs::OUTPUT;},
			_       => { _next = NextIs::NOTHING;},
		}

	}

	if arg_signature.is_empty() {
		cli_help();
		panic!("No signature !?");
	}

	println!();
	println!("- Signature\t`{}`",        arg_signature);
	println!("- Difficulty\t{} zero(s)", arg_difficulty);
	println!("- Max results\t{}",        arg_max_results);
	println!("- Leading `0`\t{}",           arg_leading0);
	println!("- Nbr threads\t{} CPU(s)", arg_threads);
	println!("- Output\t{:?} file",      arg_output);
	println!();

	let parenthesis: usize = arg_signature.find('(').expect("Valid Solidity signature (?) ");
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
		leading0   : arg_leading0,
		results    : vec![],
		max_results: arg_max_results as usize,
		output     : arg_output,
	}

}


fn main() {
	let g: Globals = init_app();
	//println!("{:?}", g);

	let g = threads_launcher( &g);
	write_file(g, "All done !");
	process::exit(0);
}



//	time cargo run s "deposit(uint256)"  z 2  l true  t 3 r 8 o tsv
//	rm *.JSON; time cargo run s "deposit(uint256)"  z 2  l true  t 3 r 8 o json; cat *.JSON
//	rm *.TSV;  time cargo run s "deposit(uint256)"  z 2  l true  t 3 r 8 o tsv;  cat *.TSV
//	rm *.CSV;  time cargo run s "deposit(uint256)"  z 2  l true  t 3 r 8 o csv;  cat *.CSV
//	rm *.XML;  time cargo run s "deposit(uint256)"  z 2  l true  t 3 r 8 o xml;  cat *.XML
//	rm *.RON;  time cargo run s "deposit(uint256)"  z 2  l true  t 3 r 8 o ron;  cat *.RON
//
//	time cargo run s "deposit(uint256)"  z 2  l true  t 14 r 12 o tsv

//	time ./select0r s "deposit(uint256)"  z 2  L true  t 14 r 15 o tsv

//	time cargo run s "aaaa(uint)"  z 2  l false  t 2
//	time cargo run s "aaaa(uint)"  z 1  l true  t 3
//	time cargo run s "deposit(uint256)"  z 2  l true  t 3 r 8 o tsv
//					s signature
//					z (nbr zero)
//					r nbr results needed
//					l looking for leading zeros in priority
//					t nbr of threads (clamp by app)
