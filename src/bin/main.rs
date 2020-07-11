extern crate clap;
use clap::{App, Arg, ArgGroup};
use recut::*;
use regex::RegexSet;


fn main() {
    let matches = App::new("Recut")
        .version("0.1")
        .about("Similar to cut but with unicode support and inferred delimiters")
        .arg(
            Arg::with_name("Bytes")
                .short("b")
                .value_name("LIST")
                .help("Specifies a range of bytes to be returned. e.g b20:-2,25, will print from the first 20 bytes to until the second to last byte, followed by the 25th byte")
                .takes_value(true)
                .validator(check_formatted_lists)
        )
        .arg(
            Arg::with_name("Characters")
                .short("c")
                .help("Specifies a range of characters which will be returned. e.g c20:-2,25, will print from the first 20 characters to until the second to last character, followed by the 25th character")
                .takes_value(true)
                .value_name("LIST")
                .validator(check_formatted_lists)
        ).arg(
            Arg::with_name("Fields")
            .short("f")
            .help("Specifies a field list to output e.g 3:-2,0  outputs 3 field until second to last field followed by the first field")
            .value_name("LIST")
            .takes_value(true)
            .validator(check_formatted_lists)
        )
        .arg(
            Arg::with_name("MatchField")
            .short("m")
            .value_name("MATCHES")
            .takes_value(true)
        )
        .group(ArgGroup::with_name("Range")
            .args(&["Bytes","Characters","Fields","MatchField"])
            .required(true)
        )
        .group(ArgGroup::with_name("DataSeperator")
            .args(&["Fields","MatchField"])
            .required(false)
        )
        .arg(
            Arg::with_name("Delimiter")
            .short("d")
            .takes_value(true)
            .requires("DataSeperator")
            .help(r#"Delimiter regex which to read the input ,fields option (-f) Or MatchField option (-m) must be used. If not present an attempt will be made to infer the delimiter"#)
        )
        .arg(
            Arg::with_name("Split")
            .short("s")
            .takes_value(true)
            .requires("n")
            .help("Like D but for string literals only,fields option (-f) or MatchField option (-m) must be used.")
            .conflicts_with("Delimiter")
        ) 
        .arg(
            Arg::with_name("NoMultiByteSplit")
            .short("n")
            .help("Prevents splitting of character bytes when -b i used")
            .requires("Bytes")
        ).arg(
            Arg::with_name("FILE")
            .index(1)
            .help("The file and (Accompanying path if neccessary ) to process standard input if empty or -- Standard input is used")
        ).after_help("Only one argument containing type LIST permitted.
        \nLIST is any number of
        \nIndex
        \nIndex:
        \n:Index
        \nIndex:Index
        \nSeperated by commas.
        \nNegative values are used to navigate from tail of the list.
        \nOmitted indeces either side of a colon are infered to be 0 (left missing) or -1 (right missing)
        \nOutput is in the same manner as list is input
        \nReptitions are allowed and will be sent to STDIO")
        .get_matches();

    let input_type = match matches.value_of("FILE") {
        Some(s) if s != "-" && s != "--" => IoType::FromFile(s.to_owned()),
        _ => IoType::FromStdIn,
    };

    let cut_type = matches
        .value_of("Bytes")
        .map(|x| CutType::Bytes(x,!matches.is_present("NoMultiByteSplit")))
        .or_else(|| {
            matches
                .value_of("Characters")
                .map(|x| CutType::Characters(x))
        })
        .or_else(|| {
            matches
                .value_of("Fields")
                .map(|x| CutType::FieldsInferDelimiter(x))
        })
        .or_else(||{
            matches.value_of("MatchField")
            .map(|x| CutType::MatchesInferDelimiter(x))
        })
        .unwrap();

    let cut_type = match cut_type {
        CutType::FieldsInferDelimiter(x) => {
            if let Some(s) = matches.value_of("Delimiter") {
                CutType::FieldsRegexDelimiter(RangeDelimiter::new(x, s))
            } else if let Some(s) = matches.value_of("Split") {
                CutType::FieldsStringDelimiter(RangeDelimiter::new(x, s))
            } else {
                CutType::FieldsInferDelimiter(x)
            }
        },
        CutType::MatchesInferDelimiter(x) =>{
            if let Some(s) = matches.value_of("Delimiter") {
                CutType::MatchesRegexDelimiter(RangeDelimiter::new(x, s))
            } else if let Some(s) = matches.value_of("Split") {
                CutType::MatchesStringDelimiter(RangeDelimiter::new(x, s))
            } else {
                CutType::MatchesInferDelimiter(x)
            }
        }
        x => x,
    };

    println!("{:?} ", cut_type);

    cut(input_type,cut_type).unwrap();
}
/**
Uses a few regexes to get rid of the most obvious errors full parsing done later
*/
fn check_formatted_lists(input:String)-> Result<(),String>{
    let fail_conditions = RegexSet::new(&[
        r"[^-:,\d]",
        r"-\D",
        r"(-|,)$",
        r":[^,]*:",
        r"-[^,:]*-",

    ]).unwrap();

    // Iterate over and collect all of the matches.
    let failures: Vec<_> = fail_conditions.matches(&input).into_iter().collect();

    if failures.is_empty(){
        Ok(())
    }
    else{
        Err(String::from("Invalid List please use the help option for details on accetped lists"))
    }

}