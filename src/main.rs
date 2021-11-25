use clap::{App, Arg};
use std::{collections::HashMap, fs, time::Instant}; 

struct ListItem {
    questions: Vec<String>,
    answers: Vec<String>,
}

fn divide_string(string: &str, places: &[usize]) -> Vec<String> {
    let mut res = Vec::new();
    let mut s = 0;
    for e in places {
        let split = &string[s..*e];
        res.push(split.trim().to_string());
        s = e + 1;
    }
    let last = &string[s..string.len()];
    res.push(last.trim().to_string());
    res
}

fn takeout_string<'a>(string: &'a str, places: &[(usize, usize)]) -> String {
    let mut res = String::new();
    for (i, c) in string.char_indices() {
        let mut skip = false;
        for p in places {
            if i >= p.0 && i <= p.1 {
                skip = true;
            }
        }
        if !skip {
            res.push(c);
        }
    }
    res
}

fn divide_slash(input: &str) -> Vec<String> {
    let mut parenthesis_depth = 0;
    let mut places = Vec::<usize>::new();
    for (n, c) in input.char_indices() {
        if c == '(' {
            parenthesis_depth += 1;
        } else if c == ')' {
            parenthesis_depth -= 1;
        } else if parenthesis_depth == 0 && c == '/' {
            places.push(n);
        }
    }
    divide_string(input, &places)
}

fn divide_parenthesis(input: &str) -> Vec<String> {
    if !input.contains('(') {
        return vec![input.to_string()];
    }
    let mut parenthesis_depth = 0;
    let mut start = 0;
    let mut takeout_parts = Vec::<(usize, usize)>::new();
    for (n, c) in input.char_indices() {
        if c == '(' {
            if parenthesis_depth == 0 {
                start = n;
            }
            parenthesis_depth += 1;
        } else if c == ')' {
            if parenthesis_depth == 1 {
                takeout_parts.push((start, n));
            }
            parenthesis_depth -= 1;
        }
    }
    let part = takeout_string(input, &takeout_parts).trim().to_string();
    vec![input.to_string(), part]
}

const MAX_DEPTH: u16 = 16;

fn recurse_parts(input: &str, slash: bool, depth: u16, keep_original: bool) -> Vec<String> {
    let mut parts = {
        if slash {
            divide_slash(input)
        } else {
            divide_parenthesis(input)
        }
    };
    let mut results = Vec::<String>::new();
    if parts.len() == 1 || depth >= MAX_DEPTH {
        results.append(&mut parts);
    } else {
        for p in parts {
            results.append(&mut recurse_parts(&p, !slash, depth + 1, keep_original));
        }
        let append_current = {
            if depth == 0 {
                keep_original
            } else {
                true
            }
        };
        if append_current {
            results.push(input.to_string());
        }
    }
    return results;
}

fn parse_pairs(input: &str, seperator: char, keep_original: bool) -> Result<Vec<ListItem>, String> {
    let mut results = Vec::<ListItem>::new();
    for line in input.lines() {
        let parts: Vec<&str> = line.split(seperator).collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid number of seperators '{}' on line: '{}'!",
                seperator, line
            ));
        }
        let question = parts[0].trim();
        let mut other = Vec::new();
        other.push(question.to_string());
        let question_parts = recurse_parts(&question, question.contains('/'), 0, keep_original);

        let answer = parts[1].trim();
        let answer_parts = recurse_parts(&answer, answer.contains('/'), 0, keep_original);

        results.push(ListItem {
            questions: question_parts,
            answers: answer_parts,
        })
    }
    Ok(results)
}

fn main() -> Result<(), String> {
    let matches = App::new("listgen")
        .version("1.0")
        .author("Rijk van Putten <contact@rijkvp.nl>")
        .about("Generates lists by parsing and combining question-answer pair syntax with slashes '/' and parenthesis '()'")
        .arg(
            Arg::with_name("input")
                .help("Set the input file(s)")
                .short("i")
                .value_name("FILES")
                .value_delimiter(" ")
                .required(true)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .help("Set the output file")
                .short("o")
                .takes_value(true)
                .value_name("FILE")
                .required(true),
        )
        .arg(
            Arg::with_name("seperator")
                .help("Character used to seperate the question-answer pairs")
                .short("s")
                .default_value("=")
                .takes_value(true)
                .value_name("SEPERATOR CHAR")
        )
        .arg(
            Arg::with_name("keep original")
                .help("Keeps the original question/answer pairs")
                .short("k")
        )
        .get_matches();

    let start_time = Instant::now();

    let input_files = matches.values_of("input").expect("No input!");
    let output_file = matches.value_of("output").expect("No output!");

    let seperator_str = matches.value_of("seperator").unwrap_or("=");
    if seperator_str.len() != 1 {
        return Err(format!("Seperator can only be 1 character while input is '{}'.", seperator_str));
    }
    let seperator = seperator_str.chars().nth(0).unwrap();
    let keep_original = matches.is_present("keep original");

    let mut input = String::new();
    for file in input_files {
        let file_str =
            fs::read_to_string(&file).map_err(|e| format!("Failed to read input: {}", e))?;
        input += &file_str;
    }

    let list = parse_pairs(&input, seperator, keep_original)?;

    let mut results = HashMap::<String, Vec<String>>::new();
    for item in list.iter() {
        for q in item.questions.iter() {
            let key = q.to_lowercase();
            if results.contains_key(&key) {
                let values = results.get_mut(&key).unwrap();
                for a in item.answers.iter() {
                    if !values.contains(&a.to_lowercase()) {
                        values.push(a.to_lowercase());
                    }
                }
            } else {
                results.insert(key, item.answers.iter().map(|a| a.to_lowercase()).collect());
            }
        }
    }

    let mut sorted_results: Vec<(String, Vec<String>)> = results
        .iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    sorted_results.sort_by(|(a, _), (b, _)| a.to_lowercase().cmp(&b.to_lowercase()));

    let mut output = String::new();
    for (key, values) in sorted_results {
        let mut line = String::new();
        line.push_str(&key);
        line.push(' ');
        line.push(seperator);
        line.push(' ');

        for (i, v) in values.iter().enumerate() {
            line.push_str(v);
            if i != values.len() - 1 {
                line.push_str(" / ");
            }
        }
        line.push('\n');
        output.push_str(&line);
    }

    fs::write(output_file, output).map_err(|e| format!("Failed to write output: {}", e))?;

    println!("Complete! Parsed length: {}, Combined length: {}, Time: {} s", list.len(), results.len(), start_time.elapsed().as_secs_f32());

    Ok(())
}
