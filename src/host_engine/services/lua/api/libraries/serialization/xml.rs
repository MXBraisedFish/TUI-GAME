use mlua::{Lua, MultiValue, Table};
use quick_xml::Reader;
use quick_xml::events::Event;

use super::{args, value};

#[derive(Debug)]
struct Node {
  name: String,
  text: String,
  children: Vec<Node>,
}

pub(super) fn install(lua: &Lua, source: &Table) -> mlua::Result<()> {
  source.raw_set(
    "xml_encode",
    lua.create_function(|_, values: MultiValue| {
      let method = "serialization.xml_encode";
      let data = value::lua_to_json(args::one(method, "t", values)?, method)?;
      let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?><root>");
      encode_value(&data, None, &mut output, method, 0)?;
      output.push_str("</root>");
      value::bounded_text(method, output)
    })?,
  )?;
  source.raw_set(
    "xml_decode",
    lua.create_function(|lua, values: MultiValue| {
      let method = "serialization.xml_decode";
      let text = value::text_argument(values, method)?;
      if text.contains("<!DOCTYPE") || text.contains("<!ENTITY") {
        return Err(args::message(
          method,
          "DTD and entity declarations are not supported",
        ));
      }
      let root = parse(&text, method)?;
      if root.name != "root" {
        return Err(args::message(method, "XML document root must be <root>"));
      }
      let data = node_content(&root, method, 0)?;
      value::json_to_lua(lua, &data, method)
    })?,
  )
}

fn encode_value(
  value: &serde_json::Value,
  name: Option<&str>,
  output: &mut String,
  method: &str,
  depth: usize,
) -> mlua::Result<()> {
  if depth > 32 {
    return Err(args::message(method, "XML value exceeds 32 levels"));
  }
  if let Some(name) = name {
    validate_element_name(name, method)?;
    output.push('<');
    output.push_str(name);
    output.push('>');
  }
  match value {
    serde_json::Value::Null => {}
    serde_json::Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
    serde_json::Value::Number(value) => output.push_str(&value.to_string()),
    serde_json::Value::String(value) => escape(value, output),
    serde_json::Value::Array(values) => {
      for value in values {
        encode_value(value, Some("item"), output, method, depth + 1)?;
      }
    }
    serde_json::Value::Object(values) => {
      for (key, value) in values {
        encode_value(value, Some(key), output, method, depth + 1)?;
      }
    }
  }
  if let Some(name) = name {
    output.push_str("</");
    output.push_str(name);
    output.push('>');
  }
  if output.len() > args::MAX_API_STRING_BYTES {
    return Err(args::message(method, "serialized output exceeds 1 MiB"));
  }
  Ok(())
}

fn parse(text: &str, method: &str) -> mlua::Result<Node> {
  let mut reader = Reader::from_str(text);
  reader.config_mut().trim_text(true);
  let mut stack: Vec<Node> = Vec::new();
  let mut root = None;
  loop {
    match reader.read_event() {
      Ok(Event::Start(event)) => {
        if event.attributes().next().is_some() {
          return Err(args::message(method, "XML attributes are not supported"));
        }
        let name = std::str::from_utf8(event.name().as_ref())
          .map_err(|_| args::message(method, "XML element names must be UTF-8"))?
          .to_string();
        validate_element_name(&name, method)?;
        if stack.len() >= 32 {
          return Err(args::message(method, "XML value exceeds 32 levels"));
        }
        stack.push(Node {
          name,
          text: String::new(),
          children: Vec::new(),
        });
      }
      Ok(Event::Empty(event)) => {
        if event.attributes().next().is_some() {
          return Err(args::message(method, "XML attributes are not supported"));
        }
        let name = std::str::from_utf8(event.name().as_ref())
          .map_err(|_| args::message(method, "XML element names must be UTF-8"))?
          .to_string();
        validate_element_name(&name, method)?;
        append_node(
          Node {
            name,
            text: String::new(),
            children: Vec::new(),
          },
          &mut stack,
          &mut root,
          method,
        )?;
      }
      Ok(Event::Text(event)) => {
        let raw = event
          .decode()
          .map_err(|_| args::message(method, "invalid XML text"))?;
        let decoded = quick_xml::escape::unescape(&raw)
          .map_err(|_| args::message(method, "invalid XML entity"))?;
        if let Some(node) = stack.last_mut() {
          node.text.push_str(&decoded);
        } else if !decoded.trim().is_empty() {
          return Err(args::message(
            method,
            "text outside the XML root is not supported",
          ));
        }
      }
      Ok(Event::CData(event)) => {
        let decoded = event
          .decode()
          .map_err(|_| args::message(method, "invalid XML CDATA"))?;
        if let Some(node) = stack.last_mut() {
          node.text.push_str(&decoded);
        }
      }
      Ok(Event::End(_)) => {
        let node = stack
          .pop()
          .ok_or_else(|| args::message(method, "unbalanced XML element"))?;
        append_node(node, &mut stack, &mut root, method)?;
      }
      Ok(Event::Decl(_)) | Ok(Event::Comment(_)) | Ok(Event::PI(_)) => {}
      Ok(Event::DocType(_)) | Ok(Event::GeneralRef(_)) => {
        return Err(args::message(
          method,
          "DTD and custom entities are not supported",
        ));
      }
      Ok(Event::Eof) => break,
      Err(_) => return Err(args::message(method, "invalid XML data")),
    }
  }
  if !stack.is_empty() {
    return Err(args::message(method, "unclosed XML element"));
  }
  root.ok_or_else(|| args::message(method, "XML document has no root element"))
}

fn append_node(
  node: Node,
  stack: &mut [Node],
  root: &mut Option<Node>,
  method: &str,
) -> mlua::Result<()> {
  if let Some(parent) = stack.last_mut() {
    if !parent.text.trim().is_empty() {
      return Err(args::message(
        method,
        "mixed XML text and child elements are not supported",
      ));
    }
    parent.children.push(node);
  } else if root.replace(node).is_some() {
    return Err(args::message(
      method,
      "XML document has multiple root elements",
    ));
  }
  Ok(())
}

fn node_content(node: &Node, method: &str, depth: usize) -> mlua::Result<serde_json::Value> {
  if depth > 32 {
    return Err(args::message(method, "XML value exceeds 32 levels"));
  }
  if node.children.is_empty() {
    return Ok(serde_json::Value::String(node.text.clone()));
  }
  if !node.text.trim().is_empty() {
    return Err(args::message(
      method,
      "mixed XML text and child elements are not supported",
    ));
  }
  if node.children.iter().all(|child| child.name == "item") {
    return node
      .children
      .iter()
      .map(|child| node_content(child, method, depth + 1))
      .collect::<mlua::Result<Vec<_>>>()
      .map(serde_json::Value::Array);
  }
  let mut object = serde_json::Map::new();
  for child in &node.children {
    if child.name == "item" || object.contains_key(&child.name) {
      return Err(args::message(
        method,
        "XML object has duplicate or reserved element names",
      ));
    }
    object.insert(child.name.clone(), node_content(child, method, depth + 1)?);
  }
  Ok(serde_json::Value::Object(object))
}

fn validate_element_name(name: &str, method: &str) -> mlua::Result<()> {
  let mut chars = name.chars();
  let valid_start = chars
    .next()
    .is_some_and(|ch| ch == '_' || ch.is_alphabetic());
  if !valid_start || chars.any(|ch| !(ch == '_' || ch == '-' || ch == '.' || ch.is_alphanumeric()))
  {
    Err(args::message(
      method,
      format!("invalid XML element name '{name}'"),
    ))
  } else {
    Ok(())
  }
}

fn escape(value: &str, output: &mut String) {
  for ch in value.chars() {
    match ch {
      '&' => output.push_str("&amp;"),
      '<' => output.push_str("&lt;"),
      '>' => output.push_str("&gt;"),
      '"' => output.push_str("&quot;"),
      '\'' => output.push_str("&apos;"),
      ch => output.push(ch),
    }
  }
}
