use sea_core::parser::{parse, ParseError};

#[test]
fn deeply_nested_expressions_parse_or_error_gracefully() {
    let mut source = String::from("Policy deep as: ");
    for _ in 0..30 {
        source.push_str("(1 + ");
    }
    source.push('1');
    for _ in 0..30 {
        source.push(')');
    }

    let result = std::panic::catch_unwind(|| parse(&source));
    match result {
        Ok(Err(_)) | Ok(Ok(_)) => {}
        Err(_) => panic!("parser panicked for 30-deep nesting"),
    }
}

#[test]
fn long_string_does_not_panic() {
    let long_val = "A".repeat(100_000);
    let source = format!("Entity \"{}\"", long_val);

    let result = std::panic::catch_unwind(|| parse(&source));
    match result {
        Ok(Err(_)) | Ok(Ok(_)) => {}
        Err(_) => panic!("parser panicked on long string input"),
    }
}

#[test]
fn import_cycle_produces_diagnostic() {
    let err = ParseError::circular_dependency(vec![
        "a.sea".to_string(),
        "b.sea".to_string(),
        "a.sea".to_string(),
    ]);
    let msg = err.to_string();
    assert!(
        msg.contains("Circular dependency"),
        "expected circular dependency message, got: {}",
        msg
    );
    assert!(
        msg.contains("a.sea -> b.sea -> a.sea"),
        "expected cycle path in message, got: {}",
        msg
    );
}

#[test]
fn large_input_does_not_panic() {
    let mut source = String::with_capacity(200_000);
    for i in 0..10_000 {
        source.push_str(&format!("Entity \"item_{}\" in domain\n", i));
    }

    let result = std::panic::catch_unwind(|| parse(&source));
    match result {
        Ok(Err(_)) | Ok(Ok(_)) => {}
        Err(_) => panic!("parser panicked on large input ({} bytes)", source.len()),
    }
}

#[test]
fn moderately_nested_policy_boolean_parses() {
    let mut source = String::from("Policy bool_nest as: ");
    for _ in 0..30 {
        source.push_str("(A and ");
    }
    source.push_str("true");
    for _ in 0..30 {
        source.push(')');
    }

    let result = std::panic::catch_unwind(|| parse(&source));
    match result {
        Ok(Err(_)) | Ok(Ok(_)) => {}
        Err(_) => panic!("parser panicked for 30-deep boolean nesting"),
    }
}

#[test]
fn many_rapid_declarations_do_not_panic() {
    let mut source = String::new();
    for i in 0..5_000 {
        source.push_str(&format!(
            "Flow \"res_{}\" from \"A\" to \"B\" quantity 1\n",
            i
        ));
    }

    let result = std::panic::catch_unwind(|| parse(&source));
    match result {
        Ok(Err(_)) | Ok(Ok(_)) => {}
        Err(_) => panic!("parser panicked on many rapid declarations"),
    }
}

#[test]
fn null_bytes_in_input_do_not_panic() {
    let source = "Entity \"test\x00payload\"";
    let result = std::panic::catch_unwind(|| parse(source));
    match result {
        Ok(Err(_)) | Ok(Ok(_)) => {}
        Err(_) => panic!("parser panicked on null bytes in input"),
    }
}

#[test]
fn extremely_long_line_does_not_panic() {
    let long_line = "X".repeat(100_000);
    let source = format!("Entity \"{}\"", long_line);

    let result = std::panic::catch_unwind(|| parse(&source));
    match result {
        Ok(Err(_)) | Ok(Ok(_)) => {}
        Err(_) => panic!("parser panicked on extremely long line"),
    }
}
