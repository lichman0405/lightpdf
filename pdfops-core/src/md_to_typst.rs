use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue, ListType},
    parse_document,
};

pub struct TypstDocument {
    pub source: String,
}

pub fn markdown_to_typst(markdown: &str, title: Option<&str>) -> TypstDocument {
    let arena = Arena::new();
    let mut opts = Options::default();
    opts.extension.table = true;
    opts.extension.strikethrough = true;
    opts.extension.tasklist = true;
    opts.extension.footnotes = true;
    opts.extension.math_dollars = true;
    let root = parse_document(&arena, markdown, &opts);
    let mut buf = String::new();
    buf.push_str("#set page(paper: \"a4\", margin: (x: 2.5cm, y: 3cm))\n");
    buf.push_str("#set text(font: \"New Computer Modern\", size: 11pt, lang: \"en\")\n");
    buf.push_str("#set par(justify: true, leading: 0.65em)\n");
    buf.push_str("#set heading(numbering: \"1.1\")\n");
    buf.push_str("#show link: underline\n");
    if let Some(t) = title {
        buf.push_str(&format!("\n#align(center)[#text(18pt, weight: \"bold\")[{}]]\n\n", escape_typst(t)));
    }
    buf.push('\n');
    render_children(root, &mut buf, false);
    TypstDocument { source: buf }
}

fn render_node<'a>(node: &'a AstNode<'a>, buf: &mut String, in_table: bool) {
    match &node.data.borrow().value {
        NodeValue::Document => render_children(node, buf, in_table),
        NodeValue::Heading(h) => {
            let prefix = "=".repeat(h.level as usize);
            buf.push('\n'); buf.push_str(&prefix); buf.push(' ');
            render_children(node, buf, false); buf.push('\n');
        }
        NodeValue::Paragraph => {
            if !in_table { buf.push('\n'); }
            render_children(node, buf, in_table);
            if !in_table { buf.push('\n'); }
        }
        NodeValue::Text(text) => { buf.push_str(&escape_typst(text)); }
        NodeValue::Strong => { buf.push('*'); render_children(node, buf, in_table); buf.push('*'); }
        NodeValue::Emph => { buf.push('_'); render_children(node, buf, in_table); buf.push('_'); }
        NodeValue::Strikethrough => { buf.push_str("#strike["); render_children(node, buf, in_table); buf.push(']'); }
        NodeValue::Code(code) => { buf.push('`'); buf.push_str(&code.literal); buf.push('`'); }
        NodeValue::CodeBlock(b) => {
            let lang = b.info.trim();
            buf.push_str("\n```"); buf.push_str(lang); buf.push('\n');
            buf.push_str(&b.literal); buf.push_str("```\n");
        }
        NodeValue::Math(math) => {
            if math.display_math {
                buf.push_str("\n$ "); buf.push_str(&math.literal); buf.push_str(" $\n");
            } else {
                buf.push('$'); buf.push_str(&math.literal); buf.push('$');
            }
        }
        NodeValue::Link(link) => {
            buf.push_str("#link(\""); buf.push_str(&link.url); buf.push_str("\")[");
            render_children(node, buf, in_table); buf.push(']');
        }
        NodeValue::Image(link) => {
            buf.push_str("\n#figure(\n  image(\""); buf.push_str(&link.url);
            buf.push_str("\"),\n  caption: ["); render_children(node, buf, in_table); buf.push_str("],\n)\n");
        }
        NodeValue::BlockQuote => {
            buf.push_str("\n#quote(block: true)[\n"); render_children(node, buf, in_table); buf.push_str("]\n");
        }
        NodeValue::ThematicBreak => { buf.push_str("\n#line(length: 100%)\n"); }
        NodeValue::List(_) => { buf.push('\n'); render_children(node, buf, in_table); }
        NodeValue::Item(_) => {
            let is_ordered = node.parent().map_or(false, |p| {
                matches!(&p.data.borrow().value, NodeValue::List(l) if l.list_type == ListType::Ordered)
            });
            buf.push_str(if is_ordered { "+ " } else { "- " });
            render_children(node, buf, in_table);
            if !buf.ends_with('\n') { buf.push('\n'); }
        }
        NodeValue::TaskItem(task) => {
            buf.push_str(if task.symbol.is_some() { "- [x] " } else { "- [ ] " });
            render_children(node, buf, in_table);
            if !buf.ends_with('\n') { buf.push('\n'); }
        }
        NodeValue::Table(nt) => {
            let cols = nt.num_columns;
            buf.push_str(&format!("\n#table(\n  columns: {cols},\n"));
            render_children(node, buf, true); buf.push_str(")\n");
        }
        NodeValue::TableRow(_) => { render_children(node, buf, true); }
        NodeValue::TableCell => { buf.push_str("  ["); render_children(node, buf, true); buf.push_str("],\n"); }
        NodeValue::SoftBreak => { buf.push(' '); }
        NodeValue::LineBreak => { buf.push_str("\\\n"); }
        NodeValue::HtmlInline(_) | NodeValue::HtmlBlock(_) => {}
        NodeValue::FootnoteDefinition(_) => {
            buf.push_str("#footnote["); render_children(node, buf, in_table); buf.push(']');
        }
        NodeValue::FootnoteReference(r) => {
            buf.push_str(&format!("#note(\"{}\")", r.name));
        }
        _ => { render_children(node, buf, in_table); }
    }
}

fn render_children<'a>(node: &'a AstNode<'a>, buf: &mut String, in_table: bool) {
    for child in node.children() { render_node(child, buf, in_table); }
}

fn escape_typst(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '#' | '@' | '<' | '>' | '&' | '~' | '\'' | '`' => { out.push('\\'); out.push(ch); }
            _ => out.push(ch),
        }
    }
    out
}