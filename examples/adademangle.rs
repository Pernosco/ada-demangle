use std::borrow::Cow;
use std::io::{self, BufRead, Write};

use ada_demangle::*;

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
    fn finish(&mut self, symbol_type: Option<SpecialSymbolType>) {
        match symbol_type {
            Some(SpecialSymbolType::TaskBody) => self.inner.push_str(" task body"),
            _ => (),
        }
    }
}

fn demangle_all<R, W>(input: &mut R, out: &mut W) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    let mut buf = vec![];

    let mut visitor = Visitor {
        inner: String::new(),
    };

    while input.read_until(b'\n', &mut buf)? > 0 {
        let nl = buf.ends_with(&[b'\n']);
        if nl {
            buf.pop();
        }
        demangle(&buf[..], &mut visitor);
        write!(out, "{}", visitor.inner)?;
        if nl {
            write!(out, "\n")?;
        }
        buf.clear();
        visitor.inner.clear();
    }

    Ok(())
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    demangle_all(&mut stdin, &mut stdout).unwrap();
}
