use regex::{Regex, RegexBuilder};

const CHARSET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+/";

fn int_2_base(x: i32, base: i32) -> String {
	let mut sign = 1;
	match x.cmp(&0) {
		std::cmp::Ordering::Less => sign = -1,
		std::cmp::Ordering::Equal => return "0".to_string(),
		std::cmp::Ordering::Greater => (),
	}
	let mut x = x * sign;
	let mut digits = Vec::new();
	while x != 0 {
		digits.push(CHARSET.chars().nth((x % base) as usize).unwrap());
		x /= base;
	}
	if sign < 0 {
		digits.push('-');
	}
	digits.reverse();
	digits.into_iter().collect()
}

fn filter_args(source: &str) -> Option<(&str, Vec<&str>, i32, i32)> {
	let regexes = [
		RegexBuilder::new(r"}\('(.*)', *(\d+), *(\d+), *'(.*)'\.split\('\|'\), *(\d+), *(.*)\)\)")
			.dot_matches_new_line(true)
			.build()
			.unwrap(),
		RegexBuilder::new(r"}\('(.*)', *(\d+), *(\d+), *'(.*)'\.split\('\|'\)")
			.dot_matches_new_line(true)
			.build()
			.unwrap(),
	];
	for regex in regexes.iter() {
		if let Some(args) = regex.captures(source) {
			let payload = args.get(1)?.as_str();
			let radix = args.get(2)?.as_str().parse::<i32>().ok()?;
			let count = args.get(3)?.as_str().parse::<i32>().ok()?;
			let symtab: Vec<&str> = args.get(4)?.as_str().split('|').collect();
			return Some((payload, symtab, radix, count));
		}
	}
	None
}

fn unpack(p: &str, a: i32, c: i32, k: Vec<&str>) -> String {
	let mut p = p.to_string();
	for i in (0..c).rev() {
		if !k[i as usize].is_empty() {
			let re = Regex::new(&format!(r"\b{}\b", int_2_base(i, a))).unwrap();
			p = re.replace_all(&p, k[i as usize]).to_string();
		}
	}
	p
}

pub fn unpack_source(source: &str) -> Option<String> {
	let (payload, symtab, radix, count) = filter_args(source)?;
	Some(unpack(payload, radix, count, symtab))
}