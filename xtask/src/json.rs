//! A tiny JSON value and pretty printer. The runner has no runtime dependencies, so it
//! writes JSON itself. `report::tests::report_conforms_to_schema` proves the output is
//! valid against `contracts/verification.report.schema.json`.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Int(i64),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn str(s: impl Into<String>) -> Json {
        Json::Str(s.into())
    }

    pub fn obj(pairs: Vec<(&str, Json)>) -> Json {
        Json::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    pub fn pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, 0);
        out.push('\n');
        out
    }

    fn write(&self, out: &mut String, depth: usize) {
        match self {
            Json::Int(i) => out.push_str(&i.to_string()),
            Json::Num(n) => {
                // JSON has no NaN or infinity. A gate that produces one has a bug,
                // and 0 with a loud failure elsewhere beats invalid JSON.
                if n.is_finite() {
                    out.push_str(&n.to_string());
                } else {
                    out.push('0');
                }
            }
            Json::Str(s) => write_string(out, s),
            Json::Arr(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    indent(out, depth + 1);
                    item.write(out, depth + 1);
                    if i + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push(']');
            }
            Json::Obj(pairs) => {
                if pairs.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push_str("{\n");
                for (i, (k, v)) in pairs.iter().enumerate() {
                    indent(out, depth + 1);
                    write_string(out, k);
                    out.push_str(": ");
                    v.write(out, depth + 1);
                    if i + 1 < pairs.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push('}');
            }
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn write_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_quotes_backslashes_and_control_chars() {
        let j = Json::str("a\"b\\c\nd\u{1}");
        assert_eq!(j.pretty(), "\"a\\\"b\\\\c\\nd\\u0001\"\n");
    }

    #[test]
    fn empty_containers_are_compact() {
        assert_eq!(Json::Arr(vec![]).pretty(), "[]\n");
        assert_eq!(Json::Obj(vec![]).pretty(), "{}\n");
    }

    #[test]
    fn non_finite_numbers_never_produce_invalid_json() {
        assert_eq!(Json::Num(f64::NAN).pretty(), "0\n");
        assert_eq!(Json::Num(f64::INFINITY).pretty(), "0\n");
    }

    #[test]
    fn nested_structure_is_indented_two_spaces() {
        let j = Json::obj(vec![("a", Json::Arr(vec![Json::Int(1), Json::Num(2.5)]))]);
        assert_eq!(j.pretty(), "{\n  \"a\": [\n    1,\n    2.5\n  ]\n}\n");
    }
}
