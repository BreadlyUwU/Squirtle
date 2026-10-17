use std::process::Command;

pub fn search_for(search: &str) -> Vec<String> {
    grab()
        .lines()
        .filter(|&line| line.contains(search))
        .map(|line| line.to_string())
        .collect()
}

fn grab() -> String {
    let lspci = Command::new("lspci").output().expect("Cannot run the 'lspci' binary.");
    String::from_utf8(lspci.stdout).expect("Weirdly enough, the output is an invalid UTF-8 sequence.")
}
