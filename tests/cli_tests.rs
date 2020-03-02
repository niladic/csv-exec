use assert_cmd::Command;

#[test]
fn test_simple_substitution() {
    let input = r#"
Id,Dir
24,example.com/a
68,example.com/b
"#
    .trim_start();

    let expected_output = r#"
Id,Dir,Result
24,example.com/a,example.com/a/24
68,example.com/b,example.com/b/68
"#
    .trim_start();

    Command::cargo_bin("csv-exec")
        .unwrap()
        .args(&["echo $1/$0"])
        .write_stdin(input)
        .assert()
        .stdout(expected_output);
}

#[test]
fn test_arg_regex() {
    let input = r#"
Id,Dir
24,example.com/a
68,example.com/b
"#
    .trim_start();

    let expected_output = r#"
Id,Dir,Result
24,example.com/a,example.com/a/24
68,example.com/b,example.com/b/68
"#
    .trim_start();

    Command::cargo_bin("csv-exec")
        .unwrap()
        .args(&["echo €1/€0", "--arg-regex", "€([0-9]+)"])
        .write_stdin(input)
        .assert()
        .stdout(expected_output);
}

#[test]
fn test_delimiter_semicolon() {
    let input = r#"
Id;Dir
24;example.com/a
68;example.com/b
"#
    .trim_start();

    let expected_output = r#"
Id;Dir;Result
24;example.com/a;example.com/a/24
68;example.com/b;example.com/b/68
"#
    .trim_start();

    Command::cargo_bin("csv-exec")
        .unwrap()
        .args(&["echo $1/$0", "-d", ";"])
        .write_stdin(input)
        .assert()
        .stdout(expected_output);
}

#[test]
fn test_delimiter_tab() {
    let input = "
Id\tDir
24\texample.com/a
68\texample.com/b
"
    .trim_start();

    let expected_output = "
Id\tDir\tResult
24\texample.com/a\texample.com/a/24
68\texample.com/b\texample.com/b/68
"
    .trim_start();

    Command::cargo_bin("csv-exec")
        .unwrap()
        .args(&["echo $1/$0", "-d", "\\t"])
        .write_stdin(input)
        .assert()
        .stdout(expected_output);

    Command::cargo_bin("csv-exec")
        .unwrap()
        .args(&["echo $1/$0", "-d", "\t"])
        .write_stdin(input)
        .assert()
        .stdout(expected_output);
}

#[test]
fn test_no_headers() {
    let input = r#"
24,example.com/a
68,example.com/b
"#
    .trim_start();

    let expected_output = r#"
24,example.com/a,example.com/a/24
68,example.com/b,example.com/b/68
"#
    .trim_start();

    Command::cargo_bin("csv-exec")
        .unwrap()
        .args(&["echo $1/$0", "--no-headers"])
        .write_stdin(input)
        .assert()
        .stdout(expected_output);
}

#[test]
fn test_new_column_name() {
    let input = r#"
Id,Dir
24,example.com/a
68,example.com/b
"#
    .trim_start();

    let expected_output = r#"
Id,Dir,A Result
24,example.com/a,example.com/a/24
68,example.com/b,example.com/b/68
"#
    .trim_start();

    Command::cargo_bin("csv-exec")
        .unwrap()
        .args(&["echo $1/$0", "--new-column-name", "A Result"])
        .write_stdin(input)
        .assert()
        .stdout(expected_output);
}
