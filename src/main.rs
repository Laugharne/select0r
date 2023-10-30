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
const FOUND: &str             = "⭐";	// "■"


#[derive(Clone)]
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
#[derive(Clone)]
#[allow(dead_code)]
struct Globals {
	signature  : String,
	part_name  : String,
	part_args  : String,
	difficulty : u32,
	nn_threads : usize,
	digit_max  : u32,
	decrease   : bool,
	results    : Vec<SignatureResult>,
	max_results: usize,
	output     : Output,
}

struct SelectorResult {
	selector    : u32,
	zero_counter: u32,
}


#[derive(Clone)]
#[derive(Debug)]
struct SignatureResult {
	signature   : String,
	selector    : u32,
	leading_zero: u32,
}


fn base64_to_string( digit: u32, value: IteratedValue) -> Result<String, std::string::FromUtf8Error> {

	// An identifier in solidity has to start with a letter, a dollar-sign or an underscore and may
	// additionally contain numbers after the first symbol.
	//
	// https://docs.soliditylang.org/en/develop/grammar.html#a4.SolidityLexer.Identifier
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


fn signature_to_selector(signature: &str, mut hasher: Sha3) -> SelectorResult {

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
/*
	let mut zero_counter: u32 = 0;
	let mut selector_u32: u32 = 0;
	for i in 0..4 {
		if selector_u8_vec[i] == 0 {
			zero_counter += 1;
		}

		selector_u32 = (selector_u32<<8) + (selector_u8_vec[i] as u32);
	}
*/
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


fn compute(g: &Globals, digit: u32, value: IteratedValue, hasher: Sha3) -> Option<SignatureResult> {
	let value64: String     = base64_to_string(digit, value).unwrap();
	let signature: String   = format!("{}_{}{}",g.part_name ,value64, g.part_args );
	let s2s: SelectorResult = signature_to_selector(&signature, hasher);
	let selector_u32: u32   = s2s.selector;
	let zero_counter: u32   = s2s.zero_counter;

	if selector_u32 == 0 {return None;}
	if zero_counter < g.difficulty {return None;}

	//println!("{:>8x}\t{}\t{:?}", selector_u32, signature, &selector_u8_vec[..4]);
	let leading_zero = get_leading_zeros(selector_u32);

	Some( SignatureResult {
		signature   : signature,
		selector    : selector_u32,
		leading_zero: leading_zero,
	})

}

fn get_leading_zeros(selector_u32: u32) -> u32 {
	let mut leading_zero: u32 = 0;
	if (selector_u32 & 0xFF000000) == 0 {
		leading_zero += 1;
		if (selector_u32 & 0x00FF0000) == 0 {
			leading_zero += 1;
			if (selector_u32 & 0x0000FF00) == 0 {
				leading_zero += 1;
			}
		}
	}
	leading_zero
}


fn thread(g: Globals, idx: IteratedValue, digit: u32, max: IteratedValue) {
	let hasher: crypto::sha3::Sha3 = crypto::sha3::Sha3::keccak256();
	let mut optimal:u32            = u32::MAX;                         // À REVOIR !
	let mut nn_results: usize      = 1;                                // À REVOIR !
	{
		let shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().unwrap();
		if let Some(last_signature) = shared.last() {
			optimal    = last_signature.selector;
			nn_results = shared.len();
		}
	}

	(idx..max).step_by(g.nn_threads).for_each( |value| {
		match compute(&g, digit, value, hasher) {
			None => {},
			Some(s) => {

				if g.decrease == true {
					if s.selector < optimal {
						optimal = s.selector;
						{
							let mut shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().unwrap();
							if let Some(last_signature) = shared.last() {

								nn_results = shared.len();

								let shared_optimal: u32 = last_signature.selector;
								if shared_optimal < optimal {
									optimal = shared_optimal;
								} else if shared_optimal > optimal {
									print!("{}", FOUND);
									shared.push( SignatureResult{
										signature   : s.signature,
										selector    : optimal,
										leading_zero: s.leading_zero,
									});
									nn_results += 1;
								}

							}// if let Some(last_signature)
						}// SHARED_RESULTS.lock()
					}
				} else {
					//println!("  [{:>08X}]\t{}", s.selector, s.signature);
					print!("{}", FOUND);
					let mut shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().unwrap();
					shared.push( SignatureResult{
						signature   : s.signature,
						selector    : optimal,
						leading_zero: s.leading_zero,
					});
					nn_results = shared.len();
				}

				if nn_results >= g.max_results as usize {
					write_file(&g);
					process::exit(0);
					//return;
				}

			}// Some()
		};// match compute()
	});// step_by(g.nn_threads).for_each(value)

}


fn threads_launcher(g: &Globals) {
	{
		let mut shared = SHARED_RESULTS.lock().unwrap();
		shared.push(SignatureResult {
			signature   : g.signature.clone(),
			selector    : u32::MAX,	//TO REWRITE
			leading_zero: 0
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
	
		println!("");

	});// for_each( digit)
	println!("\n");

}


fn write_file(g: &Globals) {
	let file_name: String = format!("select0r-{}--zero={}-max={}-decr={}-cpu={}.{:?}",
							g.signature, g.difficulty, g.max_results, g.decrease, g.nn_threads, g.output);
	let mut csv_file: Result<File, std::io::Error> = File::create(file_name);

	match csv_file {
		Ok(ref mut f) => {
			let format: &str = match g.output {
				Output::TSV  => "SELECTOR\tLEADING_ZERO\tSIGNATURE\n",
				Output::CSV  => "\"SELECTOR\",\"LEADING_ZERO\",\"SIGNATURE\"\n",
				Output::JSON => "{\n",
				Output::XML  => "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<select0r>\n",
			};
			let _ = f.write(format.as_bytes());

			let shared: std::sync::MutexGuard<'_, Vec<SignatureResult>> = SHARED_RESULTS.lock().unwrap();
			let mut line_idx: u32 = 0;
			//for line in &g.results {
			for line in shared.iter() {
				let line_csv: String = match g.output {
					Output::TSV  =>	format!("{:>08x}\t{}\t{}\n", line.selector, line.leading_zero, line.signature),
					Output::CSV  =>	format!("\"{:>08x}\",\"{}\",\"{}\"\n", line.selector, line.leading_zero, line.signature),
					Output::JSON =>	format!("\t{}{{ \"selector\":\"{:>08x}\", \"leading_zero\":\"{}\", \"signature\":\"{}\" }},\n"
						,if line_idx==0 {" "}else{","},line.selector, line.leading_zero, line.signature),
					Output::XML  =>	format!("\t<result>\n\t\t<selector>{:>08x}</selector>\n\t\t<leading_zero>{}</leading_zero>\n\t\t<signature>{}</signature>\n\t</result>\n", line.selector, line.leading_zero, line.signature),
				};
				let _ = f.write(line_csv.as_bytes());
				line_idx += 1;
			}	

			let format: &str = match g.output {
				Output::JSON => "}\n",
				Output::XML  => "</select0r>\n",
				_            => "",
			};
			let _ = f.write(format.as_bytes());

		},
		Err(_e) => panic!(),
	}

}


fn cli_help() {
	// eprintln
	// equivalent to println!() except the output goes to
	// standard err (stderr) instead of standard output (stdio)
	eprintln!(
		"\n{} - Selector Optimizer, find better function name to optimize gas cost",
		"Select0r".green().bold()
	);
	eprintln!("Usage : select0r s <function_signature string> z <number_of_zeros> r <max_results> d <decrement boolean> t <nbr_threads> o <format_ouput>");
	eprintln!("");
	eprintln!("Example 1 : select0r s \"functionName(uint256)\"  z 2  r 5  d true  t 2  o tsv");
	eprintln!("Example 2 : select0r s \"functionName2(uint)\"  z 2  r 7  d false  t 2  o json");
	eprintln!("");
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
		DECREASE,
		THREADS,
		OUTPUT,
	}

	let mut _next: NextIs = NextIs::NOTHING;

	for arg in &args {
		//println!("- {}", arg);
		match _next {
			NextIs::SIGNATURE => { arg_signature   = arg.to_string();},
			NextIs::ZERO      => { arg_difficulty  = arg.parse::<u32>().unwrap().clamp(1,3);},
			NextIs::RESULTS   => { arg_max_results = arg.parse::<u32>().unwrap().clamp(2,20);},
			NextIs::DECREASE  => { arg_decrease    = match arg.as_str() {"1"|"true"|"TRUE"=>true, "0"|"false"|"FALSE"=>false, _=>panic!("Invalid decrease value")};},
			NextIs::THREADS   => { arg_threads     = arg.parse::<usize>().unwrap().clamp( 1, num_cpus::get() as usize);},
			NextIs::OUTPUT    => { arg_output = match arg.as_str() {
									"tsv" |"TSV"|"" => Output::TSV,
									"csv" |"CSV"    => Output::CSV,
									"json"|"JSON"   => Output::JSON,
									"xml" |"XML"    => Output::XML,
									_               => panic!("Invalid output value")
								};},
			_                 => {},
		}
		_next = NextIs::NOTHING;

		match arg.as_str() {
			"s"|"S" => { _next = NextIs::SIGNATURE;},
			"z"|"Z" => { _next = NextIs::ZERO;},
			"r"|"R" => { _next = NextIs::RESULTS;},
			"d"|"D" => { _next = NextIs::DECREASE;},
			"t"|"T" => { _next = NextIs::THREADS;},
			"o"|"O" => { _next = NextIs::OUTPUT;},
			_       => { _next = NextIs::NOTHING;},
		}

	}

	if arg_signature.len() <= 0 {
		cli_help();
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
		max_results: arg_max_results as usize,
		output     : arg_output,
	}

}


fn main() {
	let g: Globals = init_app();
	//println!("{:?}", g);
	threads_launcher( &g);
	write_file(&g);
	process::exit(0);

}


//	time cargo run s "deposit(uint256)"  z 2  d true  t 3 r 8 o tsv
//	time cargo run s "deposit(uint256)"  z 2  d true  t 14 r 12 o tsv

//	time ./select0r s "deposit(uint256)"  z 2  d true  t 14 r 15 o tsv

//	time cargo run s "aaaa(uint)"  z 2  d false  t 2
//	time cargo run s "aaaa(uint)"  z 1  d true  t 3
//	time cargo run s "deposit(uint256)"  z 2  d true  t 3 r 8 o tsv
//					s signature
//					z (nbr zero)
//					r nbr results needed
//					d decrease values
//					t nbr of threads (clamp by app)

/*


action=$(yad --form --width 400 --height 300  --field="Select0r":LBL --field="":LBL --field="Signature":CE  --field="Nbr of Results":CE  --field="Nbr of zero":CB  --field="Nbr of Threads":CB --field="Ouput":CB  --field="Decrease":CHK "gtk-cancel:1"  "" "mint(address)" "4" "1\!^2\!3" "^1\!2\!3\!4\!5\!6\!7\!8\!9\!10\!11\!12\!13\!14\!15\!16" "^TSV\!CSV\!JSON\!XML" "FALSE");echo "$action"

action=$(yad --form --width 400 --height 300 \
--field="Select0r":LBL  \
--field="":LBL \
--field="Signature":CE \
--field="Nbr of Results":CE \
--field="Nbr of zero":CB \
--field="Nbr of Threads":CB \
--field="Ouput":CB \
--field="Decrease":CHK \
"gtk-cancel:1"  "" "mint(address)" "4" "1\!^2\!3" "^1\!2\!3\!4\!5\!6\!7\!8\!9\!10\!11\!12\!13\!14\!15\!16" "^TSV\!CSV\!JSON\!XML" "FALSE")
echo "$action"


result=$(yad \
--title='Select0r' \
--form --width 400 --height 300 \
--field="<b>Find better function name to optimize gas cost.</b>":LBL '' \
--field="":LBL '' \
--field="Signature" 'mint(address)' \
--field="Nbr of Results":CB '1\!2\!3\!^4\!5\!6\!7\!8\!9\!10' \
--field="Nbr of zero":CB '1\!^2\!3' \
--field="Nbr of Threads":CB '1\!^2\!3\!4\!5\!6\!7\!8\!9\!10\!11\!12\!13\!14\!15\!16' \
--field="Ouput":CB '^TSV\!CSV\!JSON\!XML' \
--field="Decrease":CHK 'FALSE' \
)
signature=$(echo "$result" | awk 'BEGIN {FS="|" } { print $3 }')
nn_result=$(echo "$result" | awk 'BEGIN {FS="|" } { print $4 }')
nn_zero=$(echo "$result" | awk 'BEGIN {FS="|" } { print $5 }')
nn_threads=$(echo "$result" | awk 'BEGIN {FS="|" } { print $6 }')
output=$(echo "$result" | awk 'BEGIN {FS="|" } { print $7 }')
decrease=$(echo "$result" | awk 'BEGIN {FS="|" } { print $8 }')
echo "$result"
echo "$signature"
echo "$nn_result"
echo "$nn_threads"
echo "$output"
echo "$decrease"

||mint(address)|4|2|2|TSV|FALSE|

*/