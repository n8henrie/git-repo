use std::collections::HashSet;
use std::env::{self, consts::OS};
use std::io::{self, Write};
use std::process::{Command, ExitStatus};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn select_from_list<T, U>(choices: &mut T) -> Result<U>
where
    T: std::iter::Iterator<Item = U>,
    U: AsRef<str>,
{
    for (idx, choice) in choices.by_ref().enumerate() {
        println!("{}: {}", idx, choice.as_ref());
    }
    let mut input = String::new();
    loop {
        print!("Choose a number from above: ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        match input.trim().parse::<usize>() {
            Ok(num) => {
                if let Some(choice) = choices.by_ref().nth(num) {
                    return Ok(choice);
                }
            }
            Err(e) => {
                writeln!(io::stderr(), "{}", e)?;
            }
        }
        input.clear();
    }
}

fn choose_remote_url<T: AsRef<str>>(urls: &HashSet<T>) -> Result<&str> {
    match urls.len() {
        0 => Err("No URL found".into()),
        1 => Ok(urls.iter().next().unwrap().as_ref()),
        _ => {
            let url = select_from_list(&mut urls.iter())?;
            Ok(url.as_ref())
        }
    }
}

fn git_output() -> Result<String> {
    Ok(String::from_utf8_lossy(
        &Command::new("git")
            .args("remote --verbose".split_whitespace())
            .output()?
            .stdout,
    )
    .into_owned())
}

fn urls_from_output<T: AsRef<str>>(output: T) -> HashSet<String> {
    output
        .as_ref()
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1).map(Into::into))
        .collect::<HashSet<_>>()
}

fn format_url<T: AsRef<str>>(url: T) -> String {
    if url.as_ref().contains(':') {
        let mut iter = url.as_ref().splitn(2, ':');
        let (user_and_domain, path) = (iter.next(), iter.next());
        let domain = user_and_domain.and_then(|x| x.splitn(2, '@').nth(1));
        match (domain, path) {
            (Some(domain), Some(path)) if !(domain.is_empty() || path.is_empty()) => {
                return format!("https://{domain}/{path}", domain = domain, path = path)
            }
            _ => (),
        }
    }
    String::from(url.as_ref())
}

fn open_url<T: AsRef<str>>(url: T) -> Result<ExitStatus> {
    let mut cmd = match OS {
        "macos" => Command::new("open"),
        "linux" => {
            let browser = env::var("BROWSER").unwrap_or_else(|_| "firefox".to_owned());
            Command::new(browser)
        }
        _ => unimplemented!("so far this only works on Mac or Linux"),
    };
    Ok(cmd.arg(url.as_ref()).status()?)
}

fn main() -> Result<()> {
    let raw_output = git_output()?;
    let urls = urls_from_output(raw_output);
    let url = choose_remote_url(&urls)?;
    open_url(format_url(url))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_choose_url() -> Result<()> {
        let mut hs = <HashSet<&str>>::new();
        hs.insert("https://n8henrie.com");
        assert_eq!(choose_remote_url(&hs)?, "https://n8henrie.com");
        Ok(())
    }

    #[test]
    fn test_urls_from_output() {
        let input = "n8henrie        git@gitlab.com:n8henrie/git-repo.git (fetch)
n8henrie        git@gitlab.com:n8henrie/git-repo.git (push)
origin  git@github.com:n8henrie/git-repo.git (fetch)
origin  git@github.com:n8henrie/git-repo.git (push)";
        let output: HashSet<String> = [
            "git@gitlab.com:n8henrie/git-repo.git",
            "git@github.com:n8henrie/git-repo.git",
        ]
        .iter()
        .cloned()
        .map(String::from)
        .collect();
        assert_eq!(urls_from_output(input), output)
    }

    #[test]
    fn test_format_url() {
        assert_eq!(
            format_url("git@github.com:n8henrie/git-repo.git"),
            "https://github.com/n8henrie/git-repo.git"
        );
        assert_eq!(
            format_url("git@gitlab.com:n8henrie/git-repo.git"),
            "https://gitlab.com/n8henrie/git-repo.git"
        );
        assert_eq!(
            format_url("https://gitlab.com/n8henrie/git-repo.git"),
            "https://gitlab.com/n8henrie/git-repo.git"
        );
    }
}
