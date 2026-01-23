#[derive(Clone)]
pub enum FormatValue<'a> {
    String(&'a str),
    Integer(i64),
}

pub type FormatArguments<'a> = &'a [(&'a str, FormatValue<'a>)];
