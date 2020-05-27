extern crate clap;
use clap::{App, Arg, SubCommand};
fn main() {
    let matches = App::new("Recut")
        .version("0.1")
        .about("Similar to cut but with unicode support and inferred delimiters")
        .arg(
            Arg::with_name("Bytes")
                .short("b")
                .value_name("LIST")
                .help("Specifies a range of bytes to be returned. e.g b20:-2,25, will print from the first 20 bytes to until the second to last byte, followed by the 25th byte")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("Characters")
                .short("c")
                .help("Specifies a range of characters which will be returned. e.g c20:-2,25, will print from the first 20 characters to until the second to last character, followed by the 25th character")
                .takes_value(true)
                .value_name("LIST")
                .conflicts_with("Bytes")
        ).arg(
            Arg::with_name("Fields")
            .short("f")
            .help("Specifies a field list to output e.g 3:-2,1  outputs 3 field until second to last field followed by the first field")
            .takes_value(true)
            .conflicts_with("Bytes")
            .conflicts_with("Characters")
        )
        .arg(
            Arg::with_name("Delimiter")
            .short("d")
            .requires("Fields")
            .help("Delimiter regex which to read the input ,fields option (-f) must be used.\n 
            If not present an attempt will be made to infer the delimiter")
        )
        .arg(
            Arg::with_name("No split of multibyte characters")
            .short("n")
            .help("prevents splitting of character bytes when -b i used")
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
        \nOut is in the same manner as list is input
        \nReptitions are allowed and will be sent to STDIO")
        .get_matches();
}
