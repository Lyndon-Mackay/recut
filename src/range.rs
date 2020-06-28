use pest::Parser;
pub enum BeginRange {
    Index(i32),
    FromStart,
}

pub enum EndRange {
    Index(i32),
    ToEnd,
}

pub enum UnExpandedIndices {
    Index(i32),
    Range(BeginRange, EndRange),
}

#[derive(Parser)]
#[grammar = "list.pest"]
pub struct ListParser;

/**
Attempts to pass the list argument if sucessfull it then converts the indices to numbers
and creates and generates a vecor of type  UnExpandedIndices
*/

/* TODO add range errors */
pub fn parse_indices(
    input: &str,
) -> std::result::Result<Vec<UnExpandedIndices>, pest::error::Error<Rule>> {
    /* Mao to act on sucessfull case transfroms parse into  */
    ListParser::parse(Rule::list, input).map(|parse| {
        parse
            .into_iter()
            .map(|parse_pair| {
                let range: Vec<_> = parse_pair.into_inner().map(|x| x.as_str()).collect();

                match range.as_slice() {
                    [index] => UnExpandedIndices::Index(index.parse().unwrap()),
                    [begin, end] if begin == &"" && end == &"" => {
                        UnExpandedIndices::Range(BeginRange::FromStart, EndRange::ToEnd)
                    }
                    [begin, end] if begin == &"" => UnExpandedIndices::Range(
                        BeginRange::FromStart,
                        EndRange::Index(end.parse().unwrap()),
                    ),
                    [begin, end] if end == &"" => UnExpandedIndices::Range(
                        BeginRange::Index(begin.parse().unwrap()),
                        EndRange::ToEnd,
                    ),
                    [begin, end] => UnExpandedIndices::Range(
                        BeginRange::Index(begin.parse().unwrap()),
                        EndRange::Index(end.parse().unwrap()),
                    ),
                    _ => unreachable!(),
                }
            })
            .collect()
    })
}
