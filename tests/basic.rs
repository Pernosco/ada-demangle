use std::borrow::Cow;

use ada_demangle::*;

#[test]
fn short_name() {
    assert!(is_short_name(b"ada_main__u00005"));
    assert!(!is_short_name(b"_ada_main"));
}

struct Visitor {
    inner: String,
}

impl<'a> DemangleVisitor<'a> for Visitor {
    fn enter_prefix(&mut self) {
        self.inner.push('[');
    }
    fn enter_ident(&mut self) {
        self.inner.push('(');
    }
    fn text(&mut self, text: Cow<'a, str>) {
        self.inner.push_str(&text);
    }
    fn exit(&mut self) {
        self.inner.push(')');
    }
    fn finish(&mut self, _: Option<SpecialSymbolType>) {
    }
}

fn check(input: &[u8], output: &str) {
    let mut visitor = Visitor {
        inner: String::new(),
    };
    assert_eq!(demangle(input, &mut visitor), Some(()));
    assert_eq!(
        visitor.inner,
        output,
        "For input {}",
        String::from_utf8_lossy(input)
    );
}

fn check_none(input: &[u8]) {
    let mut visitor = Visitor {
        inner: String::new(),
    };
    assert_eq!(demangle(input, &mut visitor), None);
}

#[test]
fn names() {
    check(b"_ada_main", "(main)");
    check(
        b"module__pcontrolled__l2",
        "[(module).)[(pcontrolled).)(l2)",
    );
    check(b"module__square__2", "[(module).)(square)");
    check(
        b"ada__exceptions__exception_traces__last_chance_handlerXn",
        "[(ada).)[(exceptions).)[(exception_traces).)(last_chance_handler)",
    );
    check(
        b"ada_main__finalize_library__B_4__reraise_library_exception_if_any",
        "[(ada_main).)[(finalize_library).)(reraise_library_exception_if_any)",
    );
    check(
        b"ada_main__adafinal__s_stalib_adafinal",
        "[(ada_main).)[(adafinal).)(s_stalib_adafinal)",
    );
    check(b"Oeq", "(=)");
    check_none(b"B53b");
}
