use std::collections::BTreeMap;

use serde::Serialize;
use syn::visit::Visit;

#[derive(Debug, Serialize, PartialEq)]
pub struct Message {
    description: String,
    #[serde(rename = "defaultMessage")]
    default_message: String,
}

pub struct FormatMessageExtractor {
    pub on_duplicate_message: Box<dyn Fn(String, String, String)>,
    pub messages: BTreeMap<String, Message>,
}

impl FormatMessageExtractor {
    pub fn new(on_duplicate: Box<dyn Fn(String, String, String)>) -> Self {
        Self {
            on_duplicate_message: on_duplicate,
            messages: BTreeMap::new(),
        }
    }

    pub fn extract_file(&mut self, content: &str) -> Result<(), syn::Error> {
        let syntax = syn::parse_file(content)?;

        self.visit_file(&syntax);

        Ok(())
    }
}

impl<'ast> syn::visit::Visit<'ast> for FormatMessageExtractor {
    fn visit_expr_macro(&mut self, i: &'ast syn::ExprMacro) {
        let macro_name = &i.mac.path.segments.last().unwrap().ident;

        // Try to parse the macro body as comma-separated expressions
        let args = i.mac.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::token::Comma>::parse_terminated,
        );

        // If parsing succeeds, visit each argument to handle nested macros
        if let Ok(ref args) = args {
            for arg in args {
                self.visit_expr(arg);
            }
        }

        if macro_name == "format_message" {
            if let Ok(args) = args {
                // expected: (formatter, message_id, description, default_message, args) or (formatter, message_id, description, default_message)
                if args.len() == 4 || args.len() == 5 {
                    let message_id = extract_string(&args[1]);
                    let description = extract_string(&args[2]);
                    let default_message = extract_string(&args[3]);

                    if let (Some(id), Some(description), Some(default_message)) =
                        (message_id, description, default_message)
                    {
                        if let Some(existing) = self.messages.get(&id) {
                            if existing.default_message != default_message {
                                (self.on_duplicate_message)(
                                    id.clone(),
                                    existing.default_message.clone(),
                                    default_message.clone(),
                                );
                            }
                        }

                        self.messages.insert(
                            id,
                            Message {
                                description,
                                default_message,
                            },
                        );
                    }
                }
            }
        }

        // Note: We don't call the default visit_expr_macro since we've handled
        // visiting the arguments
    }
}

fn extract_string(expr: &syn::Expr) -> Option<String> {
    if let syn::Expr::Lit(lit) = expr {
        if let syn::Lit::Str(s) = &lit.lit {
            return Some(s.value());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test_normal() {
        let mut extractor = FormatMessageExtractor::new(Box::new(|_, _, _| {}));
        let content = r###"fn main() { let _ = format_message!(formatter, "standalone", "A test message", "Hello world", args); }"###;
        extractor.extract_file(content).unwrap();
        assert_eq!(extractor.messages.len(), 1);
        assert_eq!(
            extractor.messages["standalone"],
            Message {
                description: "A test message".to_string(),
                default_message: "Hello world".to_string()
            }
        );
    }

    #[test]
    fn test_comma() {
        let mut extractor = FormatMessageExtractor::new(Box::new(|_, _, _| {}));
        let content = r###"fn main() { let _ = format_message!(formatter, "standalone", "A test message", "Hello, world", args); }"###;
        extractor.extract_file(content).unwrap();
        assert_eq!(extractor.messages.len(), 1);
        assert_eq!(
            extractor.messages["standalone"],
            Message {
                description: "A test message".to_string(),
                default_message: "Hello, world".to_string()
            }
        );
    }

    #[test]
    fn test_nested_once() {
        let mut extractor = FormatMessageExtractor::new(Box::new(|_, _, _| {}));
        let content = r###"fn main() { let _ = format!("{}", format_message!(formatter, "nested1", "A test message", "Nested message", args)); }"###;
        extractor.extract_file(content).unwrap();
        assert_eq!(extractor.messages.len(), 1);
        assert_eq!(
            extractor.messages["nested1"],
            Message {
                description: "A test message".to_string(),
                default_message: "Nested message".to_string()
            }
        );
    }

    #[test]
    fn test_nested_twice() {
        let mut extractor = FormatMessageExtractor::new(Box::new(|_, _, _| {}));
        let content = r###"fn main() { let _ = format!("{}", format!("{}", format_message!(formatter, "nested2", "A test message", "Double nested", args))); }"###;
        extractor.extract_file(content).unwrap();
        assert_eq!(extractor.messages.len(), 1);
        assert_eq!(
            extractor.messages["nested2"],
            Message {
                description: "A test message".to_string(),
                default_message: "Double nested".to_string()
            }
        );
    }

    #[test]
    fn test_duplicated_message_id() {
        let call_count = Rc::new(RefCell::new(0));
        let callback = {
            let call_count = call_count.clone();
            move |id: String, old: String, new: String| {
                *call_count.borrow_mut() += 1;
                assert_eq!(id, "duplicate");
                assert_eq!(old, "First message");
                assert_eq!(new, "Second message");
            }
        };
        let mut extractor = FormatMessageExtractor::new(Box::new(callback));
        let content = r###"fn main() { let _ = format_message!(formatter, "duplicate", "A test message", "First message", args); let _ = format_message!(formatter, "duplicate", "A test message", "Second message", args); }"###;
        extractor.extract_file(content).unwrap();
        assert_eq!(*call_count.borrow(), 1);
        assert_eq!(extractor.messages.len(), 1);
        assert_eq!(
            extractor.messages["duplicate"],
            Message {
                description: "A test message".to_string(),
                default_message: "Second message".to_string()
            }
        );
    }

    #[test]
    fn test_standalone_macro() {
        // This is a statement macro, not an expression macro
        let mut extractor = FormatMessageExtractor::new(Box::new(|_, _, _| {}));
        let content = r###"fn main() { format_message!(formatter, "standalone", "A test message", "Hello world", args); }"###;
        extractor.extract_file(content).unwrap();
        assert_eq!(extractor.messages.len(), 0);
    }
}
