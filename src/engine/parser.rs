use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::multispace0,
    combinator::{map, opt},
    multi::separated_list1,
    sequence::delimited,
    IResult,
    Parser,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Redirection {
    Stdout(String),
    Stderr(String),
    Append(String),
    StderrToStdout,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Command {
    pub args: Vec<String>,
    pub redirections: Vec<Redirection>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pipeline {
    pub commands: Vec<Command>,
    pub background: bool,
}

fn parse_argument(input: &str) -> IResult<&str, String> {
    alt((
        // Quoted string
        map(
            delimited(tag("\""), is_not("\""), tag("\"")),
            |s: &str| s.to_string(),
        ),
        // Simple word
        map(is_not(" |><&"), |s: &str| s.to_string()),
    )).parse(input)
}

fn parse_redirection(input: &str) -> IResult<&str, Redirection> {
    alt((
        map(tag("2>&1"), |_| Redirection::StderrToStdout),
        map((tag(">>"), multispace0, parse_argument), |(_, _, file)| {
            Redirection::Append(file)
        }),
        map((tag("2>"), multispace0, parse_argument), |(_, _, file)| {
            Redirection::Stderr(file)
        }),
        map((tag(">"), multispace0, parse_argument), |(_, _, file)| {
            Redirection::Stdout(file)
        }),
    )).parse(input)
}

fn parse_single_command(input: &str) -> IResult<&str, Command> {
    let (input, _) = multispace0(input)?;
    let mut current_input = input;
    let mut args = Vec::new();
    let mut redirections = Vec::new();

    while !current_input.is_empty() && !current_input.starts_with('|') && !current_input.starts_with('&') {
        let (next_input, _) = multispace0(current_input)?;
        current_input = next_input;

        if current_input.is_empty() || current_input.starts_with('|') || current_input.starts_with('&') {
            break;
        }

        if let Ok((next_input, red)) = parse_redirection(current_input) {
            redirections.push(red);
            current_input = next_input;
        } else if let Ok((next_input, arg)) = parse_argument(current_input) {
            args.push(arg);
            current_input = next_input;
        } else {
            break;
        }
    }

    Ok((current_input, Command { args, redirections }))
}

pub fn parse_pipeline(input: &str) -> IResult<&str, Pipeline> {
    let (input, commands) = separated_list1((multispace0, tag("|"), multispace0), parse_single_command).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, background_opt) = opt(tag("&")).parse(input)?;
    
    Ok((input, Pipeline { commands, background: background_opt.is_some() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let input = "ls -la";
        let (_, pipeline) = parse_pipeline(input).unwrap();
        assert_eq!(pipeline.commands.len(), 1);
        assert_eq!(pipeline.commands[0].args, vec!["ls", "-la"]);
    }

    #[test]
    fn test_parse_pipeline() {
        let input = "ls | grep rust | wc -l";
        let (_, pipeline) = parse_pipeline(input).unwrap();
        assert_eq!(pipeline.commands.len(), 3);
        assert_eq!(pipeline.commands[0].args, vec!["ls"]);
        assert_eq!(pipeline.commands[1].args, vec!["grep", "rust"]);
        assert_eq!(pipeline.commands[2].args, vec!["wc", "-l"]);
    }

    #[test]
    fn test_parse_quoted_arg() {
        let input = "grep \"hello world\" file.txt";
        let (_, pipeline) = parse_pipeline(input).unwrap();
        assert_eq!(pipeline.commands[0].args, vec!["grep", "hello world", "file.txt"]);
    }

    #[test]
    fn test_parse_redirections() {
        let input = "ls > out.txt 2> err.txt";
        let (_, pipeline) = parse_pipeline(input).unwrap();
        assert_eq!(pipeline.commands[0].args, vec!["ls"]);
        assert_eq!(pipeline.commands[0].redirections.len(), 2);
        assert_eq!(
            pipeline.commands[0].redirections[0],
            Redirection::Stdout("out.txt".to_string())
        );
        assert_eq!(
            pipeline.commands[0].redirections[1],
            Redirection::Stderr("err.txt".to_string())
        );
    }
}
