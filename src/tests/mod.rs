// TODO: Refactor to include test name outputs

#[cfg(test)]
mod tests {
    use std::fs::{read_dir, read_to_string, DirEntry};
    use std::process::Command;

    #[test]
    fn execute_tests() {
        let cases = read_dir("./src/tests/cases").unwrap();

        let mut errors = vec![];
        for case in cases.into_iter() {
            let case = case.unwrap();
            let name = case.path().display().to_string();
            if name.contains("~") {
                continue;
            }

            match run_test(case) {
                Ok(_) => (),
                Err(msg) => {
                    errors.push(msg);
                    break;
                }
            }
        }

        if !errors.is_empty() {
            panic!("Errors:\n\n{}", errors.join("\n\n"));
        }
    }

    fn run_test(file: DirEntry) -> Result<(), String> {
        let contents = read_to_string(file.path()).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        let mut test_code = vec![];

        let mut idx = None;
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("--- Test") {
                continue;
            }
            if line.starts_with("--- Expected") {
                idx = Some(i);
                break;
            }
            test_code.push(line.clone());
        }

        let idx = idx.unwrap_or_else(|| {
            panic!(
                "{:?}: No expected section in test case definition",
                file.file_name()
            )
        });

        let mut expected_output = vec![];

        for line in &lines[idx + 1..] {
            if !line.is_empty() {
                expected_output.push(*line);
            }
        }

        let input = test_code.join("\n");

        let output = Command::new("cargo")
            .arg("run")
            .arg("e")
            .arg(input)
            .output()
            .unwrap();
        let lines: Vec<&str> = std::str::from_utf8(output.stdout.as_slice())
            .unwrap()
            .lines()
            .collect();
        if !(lines.len() == expected_output.len() || lines.len() == expected_output.len() + 1) {
            return Err(format!(
                "{:#?}: output length does not match expected output: {} != {}\nFull output:\n{}",
                file.file_name(),
                lines.len(),
                expected_output.len(),
                lines.join("\n"),
            ));
        }

        for (i, expected) in expected_output.iter().enumerate() {
            if lines[i] != (*expected).trim() {
                return Err(format!(
                    "{:#?}: {} != {}\nFull output:\n{}",
                    file.file_name(),
                    lines[i],
                    expected,
                    lines.join("\n"),
                ));
            }
        }

        Ok(())
    }
}
