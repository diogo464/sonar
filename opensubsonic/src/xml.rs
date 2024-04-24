use std::io::Write as _;

pub trait XmlSerialize {
    fn serialize(&self, xml: &mut Xml);
}

#[derive(Debug)]
pub struct Xml {
    buffer: Vec<u8>,
    tag_stack: Vec<&'static str>,
}

pub fn new() -> Xml {
    Xml {
        buffer: Default::default(),
        tag_stack: Default::default(),
    }
}
pub fn finish(xml: Xml) -> String {
    String::from_utf8(xml.buffer).expect("invalid utf-8 in xml")
}
pub fn serialize<T>(v: &T) -> String
where
    T: XmlSerialize,
{
    let mut xml = new();
    v.serialize(&mut xml);
    finish(xml)
}
pub fn elem_begin_open(xml: &mut Xml, element: &'static str) {
    if !xml.tag_stack.is_empty() {
        let _ = writeln!(xml.buffer);
    }
    for _ in 0..xml.tag_stack.len() {
        let _ = write!(xml.buffer, "\t");
    }
    xml.tag_stack.push(element);
    let _ = write!(xml.buffer, "<{}", element);
}
pub fn elem_begin_close(xml: &mut Xml) {
    let _ = write!(xml.buffer, ">");
}
pub fn elem_begin_close_end(xml: &mut Xml) {
    xml.tag_stack.pop().expect("empty tag stack");
    let _ = write!(xml.buffer, " />");
}
pub fn attr(xml: &mut Xml, attr: &str, value: &impl std::fmt::Display) {
    let _ = write!(xml.buffer, " {}=\"", attr);

    // TODO: improve clones?
    let mut value = format!("{}", value);
    value = value.replace('&', "&amp;");
    value = value.replace('"', "&quot;");
    value = value.replace('<', "&lt;");
    value = value.replace('<', "&gt;");

    let _ = write!(xml.buffer, "{}", value);
    let _ = write!(xml.buffer, "\"");
}
pub fn elem_end(xml: &mut Xml) {
    let element = xml.tag_stack.pop().expect("empty tag stack");
    // for _ in 0..xml.tag_stack.len() {
    //     let _ = write!(xml.buffer, "\t");
    // }
    let _ = write!(xml.buffer, "</{}>", element);
}
pub fn attr_opt(xml: &mut Xml, attr_: &str, value: &Option<impl std::fmt::Display>) {
    if let Some(value) = value {
        attr(xml, attr_, value);
    }
}
pub fn body_display(xml: &mut Xml, body: &impl std::fmt::Display) {
    let _ = write!(xml.buffer, "{}", body);
}
