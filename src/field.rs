use pest::Parser;
#[derive(Parser)]
#[grammar = "line.pest"]
pub struct LineParser;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LastRule {
    Data,
    QuotedData,
    None,
}

pub fn split_line_quotes(line: &str, delimiter: &str) -> Vec<String> {
    LineParser::parse(Rule::line, line)
        .unwrap()
        .into_iter()
        .next() //one matching the first line at a time
        .unwrap()
        .into_inner()
        .into_iter()
        .fold(
            (vec![], LastRule::None), //lastrule for state and vec for output
            |(mut split_line, last_rule), inner| match inner.as_rule() {
                Rule::data => {
                    let split_data = inner.as_str().split(delimiter);

                    let mut result = if last_rule != LastRule::None {
                        split_data
                            .skip(1) // first field would have been in quoted data
                            .map(|x| x.trim().to_owned())
                            .collect::<Vec<_>>()
                    } else {
                        split_data.map(|x| x.trim().to_owned()).collect::<Vec<_>>()
                    };
                    split_line.append(&mut result);
                    (split_line, LastRule::Data)
                }
                Rule::quoted_data => {
                    if last_rule == LastRule::Data {
                        split_line = split_line.split_last().unwrap().1.to_owned();
                        // remove empty filed from last iteration as current field is its actua contents
                    }
                    split_line.push(inner.as_str().to_owned());
                    (split_line, LastRule::QuotedData)
                }
                _ => unreachable!(),
            },
        )
        .0
}
