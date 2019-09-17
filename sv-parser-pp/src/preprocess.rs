use crate::range::Range;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use sv_parser_error::{Error, ErrorKind};
use sv_parser_parser::{pp_parser, Span, SpanInfo};
use sv_parser_syntaxtree::{
    IncludeCompilerDirective, Locate, NodeEvent, RefNode, TextMacroDefinition,
};

#[derive(Debug)]
pub struct PreprocessedText {
    text: String,
    origins: BTreeMap<Range, Origin>,
}

#[derive(Debug)]
pub struct Origin {
    range: Range,
    origin_path: PathBuf,
    origin_range: Range,
}

impl PreprocessedText {
    fn new() -> Self {
        PreprocessedText {
            text: String::new(),
            origins: BTreeMap::new(),
        }
    }

    fn push<T: AsRef<Path>>(&mut self, s: &str, origin_path: T, origin_range: Range) {
        let base = self.text.len();
        self.text.push_str(s);

        let range = Range::new(base, base + s.len());
        let origin = Origin {
            range,
            origin_path: PathBuf::from(origin_path.as_ref()),
            origin_range,
        };
        self.origins.insert(range, origin);
    }

    fn merge(&mut self, other: PreprocessedText) {
        let base = self.text.len();
        self.text.push_str(&other.text);
        for (mut range, mut origin) in other.origins {
            range.offset(base);
            origin.range.offset(base);
            self.origins.insert(range, origin);
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn origin(&self, pos: usize) -> Option<(&PathBuf, usize)> {
        let origin = self.origins.get(&Range::new(pos, pos + 1));
        if let Some(origin) = origin {
            let ret_pos = pos - origin.range.begin + origin.origin_range.begin;
            Some((&origin.origin_path, ret_pos))
        } else {
            None
        }
    }
}

pub fn preprocess<T: AsRef<Path>, U: AsRef<Path>>(
    path: T,
    pre_defines: &HashMap<String, Option<(TextMacroDefinition, PathBuf)>>,
    include_paths: &[U],
) -> Result<PreprocessedText, Error> {
    let f = File::open(path.as_ref())?;
    let mut reader = BufReader::new(f);
    let mut s = String::new();
    reader.read_to_string(&mut s)?;

    let mut skip = false;
    let mut skip_nodes = vec![];
    let mut defines = HashMap::new();

    for (k, v) in pre_defines {
        defines.insert(k.clone(), v.clone());
    }

    let span = Span::new_extra(&s, SpanInfo::default());
    let (_, pp_text) = pp_parser(span).map_err(|_| ErrorKind::Parse)?;

    let mut ret = PreprocessedText::new();

    for n in pp_text.into_iter().event() {
        match n {
            NodeEvent::Enter(RefNode::ResetallCompilerDirective(_)) if !skip => {
                defines.clear();
            }
            NodeEvent::Enter(RefNode::UndefineCompilerDirective(x)) if !skip => {
                let (_, _, ref name) = x.nodes;
                let id = identifier((&name.nodes.0).into(), &s).unwrap();
                defines.remove(&id);
            }
            NodeEvent::Enter(RefNode::UndefineallCompilerDirective(_)) if !skip => {
                defines.clear();
            }
            NodeEvent::Enter(RefNode::SourceDescriptionNotDirective(x)) if !skip => {
                let locate: Locate = x.try_into().unwrap();
                let range = Range::new(locate.offset, locate.offset + locate.len);
                ret.push(locate.str(&s), path.as_ref(), range);
            }
            NodeEvent::Enter(RefNode::IfdefDirective(x)) if !skip => {
                let (_, _, ref ifid, ref ifbody, ref elsif, ref elsebody, _, _) = x.nodes;
                let ifid = identifier(ifid.into(), &s).unwrap();
                let mut hit = false;
                if defines.contains_key(&ifid) {
                    hit = true;
                } else {
                    skip_nodes.push(ifbody.into());
                }

                for x in elsif {
                    let (_, _, ref elsifid, ref elsifbody) = x;
                    let elsifid = identifier(elsifid.into(), &s).unwrap();
                    if hit {
                        skip_nodes.push(elsifbody.into());
                    } else if defines.contains_key(&elsifid) {
                        hit = true;
                    } else {
                        skip_nodes.push(elsifbody.into());
                    }
                }

                if let Some(elsebody) = elsebody {
                    let (_, _, ref elsebody) = elsebody;
                    if hit {
                        skip_nodes.push(elsebody.into());
                    }
                }
            }
            NodeEvent::Enter(RefNode::IfndefDirective(x)) if !skip => {
                let (_, _, ref ifid, ref ifbody, ref elsif, ref elsebody, _, _) = x.nodes;
                let ifid = identifier(ifid.into(), &s).unwrap();
                let mut hit = false;
                if !defines.contains_key(&ifid) {
                    hit = true;
                } else {
                    skip_nodes.push(ifbody.into());
                }

                for x in elsif {
                    let (_, _, ref elsifid, ref elsifbody) = x;
                    let elsifid = identifier(elsifid.into(), &s).unwrap();
                    if hit {
                        skip_nodes.push(elsifbody.into());
                    } else if defines.contains_key(&elsifid) {
                        hit = true;
                    } else {
                        skip_nodes.push(elsifbody.into());
                    }
                }

                if let Some(elsebody) = elsebody {
                    let (_, _, ref elsebody) = elsebody;
                    if hit {
                        skip_nodes.push(elsebody.into());
                    }
                }
            }
            NodeEvent::Enter(RefNode::TextMacroDefinition(x)) if !skip => {
                let (_, _, ref name, _) = x.nodes;
                let id = identifier((&name.nodes.0).into(), &s).unwrap();
                defines.insert(id, Some((x.clone(), PathBuf::from(path.as_ref()))));
            }
            NodeEvent::Enter(RefNode::IncludeCompilerDirective(x)) if !skip => {
                let path = match x {
                    IncludeCompilerDirective::DoubleQuote(x) => {
                        let (_, _, ref literal) = x.nodes;
                        let (locate, _) = literal.nodes;
                        locate.str(&s).trim_matches('"')
                    }
                    IncludeCompilerDirective::AngleBracket(x) => {
                        let (_, _, ref literal) = x.nodes;
                        let (locate, _) = literal.nodes;
                        locate.str(&s).trim_start_matches('<').trim_end_matches('>')
                    }
                };
                let mut path = PathBuf::from(path);
                if path.is_relative() {
                    if !path.exists() {
                        for include_path in include_paths {
                            let new_path = include_path.as_ref().join(&path);
                            if new_path.exists() {
                                path = new_path;
                                break;
                            }
                        }
                    }
                }
                let include = preprocess(path, &defines, include_paths)?;
                ret.merge(include);
            }
            NodeEvent::Enter(RefNode::TextMacroUsage(x)) if !skip => {
                let (_, ref name, ref args) = x.nodes;
                let id = identifier((&name.nodes.0).into(), &s).unwrap();

                let mut actual_args = Vec::new();
                if let Some(args) = args {
                    let (_, ref args, _) = args.nodes;
                    let (ref args,) = args.nodes;
                    for arg in args.contents() {
                        let (ref arg,) = arg.nodes;
                        let arg: Locate = arg.try_into().unwrap();
                        let arg = arg.str(&s);
                        actual_args.push(arg);
                    }
                }

                let define = defines.get(&id);
                if let Some(Some((define, define_path))) = define {
                    let (_, _, ref proto, ref text) = define.nodes;

                    let mut arg_names = Vec::new();
                    let mut defaults = Vec::new();
                    let (_, ref args) = proto.nodes;
                    if let Some(args) = args {
                        let (_, ref args, _) = args.nodes;
                        let (ref args,) = args.nodes;
                        for arg in args.contents() {
                            let (ref arg, ref default) = arg.nodes;
                            let (ref arg, _) = arg.nodes;
                            let arg = arg.str(&s);

                            let default = if let Some((_, x)) = default {
                                let x: Locate = x.try_into().unwrap();
                                let x = x.str(&s);
                                Some(x)
                            } else {
                                None
                            };

                            arg_names.push(arg);
                            defaults.push(default);
                        }
                    }

                    let mut arg_map = HashMap::new();
                    for (i, arg) in arg_names.iter().enumerate() {
                        let value = if let Some(actual_arg) = actual_args.get(i) {
                            actual_arg
                        } else {
                            if let Some(default) = defaults.get(i).unwrap() {
                                default
                            } else {
                                unimplemented!();
                            }
                        };
                        arg_map.insert(String::from(*arg), value);
                    }

                    if let Some(text) = text {
                        let text: Locate = text.try_into().unwrap();
                        let range = Range::new(text.offset, text.offset + text.len);
                        let text = text.str(&s);
                        let mut replaced = String::from("");
                        for text in split_text(text) {
                            if let Some(value) = arg_map.get(&text) {
                                replaced.push_str(**value);
                            } else {
                                replaced.push_str(&text.replace("``", ""));
                            }
                        }
                        ret.push(&replaced, define_path, range);
                    } else {
                        unimplemented!();
                    }
                } else if let Some(_) = define {
                    unimplemented!();
                } else {
                    unimplemented!();
                }
            }
            NodeEvent::Enter(x) => {
                if skip_nodes.contains(&x) {
                    skip = true;
                }
            }
            NodeEvent::Leave(x) => {
                if skip_nodes.contains(&x) {
                    skip = false;
                }
            }
        }
    }

    Ok(ret)
}

fn identifier(node: RefNode, s: &str) -> Option<String> {
    for x in node {
        match x {
            RefNode::SimpleIdentifier(x) => {
                let x: Locate = x.nodes.0.try_into().unwrap();
                return Some(String::from(x.str(s)));
            }
            RefNode::EscapedIdentifier(x) => {
                let x: Locate = x.nodes.0.try_into().unwrap();
                return Some(String::from(x.str(s)));
            }
            _ => (),
        }
    }
    None
}

fn split_text(s: &str) -> Vec<String> {
    let mut is_ident = false;
    let mut is_ident_prev;
    let mut x = String::from("");
    let mut ret = vec![];
    for c in s.chars() {
        is_ident_prev = is_ident;
        is_ident = c.is_ascii_alphanumeric() | (c == '_');

        if is_ident != is_ident_prev {
            ret.push(x);
            x = String::from("");
        }

        x.push(c);
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_testcase(s: &str) -> String {
        format!(
            "{}/testcases/{}",
            env::var("CARGO_MANIFEST_DIR").unwrap(),
            s
        )
    }

    #[test]
    fn test1() {
        let ret = preprocess(get_testcase("test1.sv"), &HashMap::new(), &[] as &[String]).unwrap();
        assert_eq!(
            ret.text(),
            r##"module and_op (a, b, c);
output a;
input b, c;

and a1 (a,b,c);
endmodule
"##
        );
        assert_eq!(
            ret.origin(10).unwrap().0,
            &PathBuf::from(get_testcase("test1.sv"))
        );
        assert_eq!(ret.origin(10).unwrap().1, 10);
        assert_eq!(ret.origin(50).unwrap().1, 98);
        assert_eq!(ret.origin(70).unwrap().1, 125);
    }

    #[test]
    fn test1_predefine() {
        let mut defines = HashMap::new();
        defines.insert(String::from("behavioral"), None);
        let ret = preprocess(get_testcase("test1.sv"), &defines, &[] as &[String]).unwrap();
        assert_eq!(
            ret.text(),
            r##"module and_op (a, b, c);
output a;
input b, c;

wire a = b & c;
endmodule
"##
        )
    }

    #[test]
    fn test2() {
        let include_paths = [get_testcase("")];
        let ret = preprocess(get_testcase("test2.sv"), &HashMap::new(), &include_paths).unwrap();
        assert_eq!(
            ret.text(),
            r##"module and_op (a, b, c);
output a;
input b, c;

and a1 (a,b,c);
endmodule
"##
        );
        assert_eq!(
            ret.origin(10).unwrap().0,
            &PathBuf::from(get_testcase("test2.sv"))
        );
        assert_eq!(ret.origin(10).unwrap().1, 10);
        assert_eq!(
            ret.origin(50).unwrap().0,
            &PathBuf::from(get_testcase("test2.svh"))
        );
        assert_eq!(ret.origin(50).unwrap().1, 73);
        assert_eq!(
            ret.origin(70).unwrap().0,
            &PathBuf::from(get_testcase("test2.sv"))
        );
        assert_eq!(ret.origin(70).unwrap().1, 52);
    }

    #[test]
    fn test3() {
        let ret = preprocess(get_testcase("test3.sv"), &HashMap::new(), &[] as &[String]).unwrap();
        assert_eq!(
            ret.text(),
            r##"

module a ();

  \
  assign a_0__x = a[0].x; \
  assign a_0__y = a[0].y;
  \
  assign a_1__x = a[1].x; \
  assign a_1__y = a[1].y;

endmodule
"##
        );
    }
}