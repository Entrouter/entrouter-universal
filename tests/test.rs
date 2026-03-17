#[test]
fn entrouter_nightmare_payload() {
    use entrouter_universal::{Envelope, Guardian, encode_str, decode_str};

    // The most evil string possible
    let nightmare = r#"{"token":"000001739850123456-000004521890000-a3f1b2-user_abc\"123\"","data":{"nested":"val\\ue","sql":"'; DROP TABLE users; --","newline":"line1\nline2\ttabbed","unicode":"héllo wörld 🔥"}}"#;

    // Test 1 — raw round trip
    let encoded = encode_str(nightmare);
    let decoded = decode_str(&encoded).unwrap();
    assert_eq!(nightmare, decoded, "raw round trip failed");

    // Test 2 — envelope through JSON serialisation (simulates HTTP → Redis → Postgres)
    let env = Envelope::wrap(nightmare);
    let as_json = env.to_json().unwrap();
    let from_json = Envelope::from_json(&as_json).unwrap();
    let result = from_json.unwrap_verified().unwrap();
    assert_eq!(nightmare, result, "envelope pipeline failed");

    // Test 3 — guardian catches a mutation
    let mut g = Guardian::new(nightmare);
    g.checkpoint("after_json", &encode_str(nightmare));        // intact
    g.checkpoint("mutated_layer", &encode_str("tampered!"));   // broken
    assert!(!g.is_intact(), "guardian should detect mutation");
    assert_eq!(g.first_violation().unwrap().layer, "mutated_layer");

    // Test 4 — guardian passes clean pipeline
    let mut g2 = Guardian::new(nightmare);
    let encoded = g2.encoded().to_string();
    g2.checkpoint("http",     &encoded);
    g2.checkpoint("json",     &encoded);
    g2.checkpoint("redis",    &encoded);
    g2.checkpoint("postgres", &encoded);
    g2.assert_intact(); // panics if anything broke
}
