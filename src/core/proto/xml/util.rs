//! Tools for handling XML from AWS with helper functions for testing.
//!
//! Wraps an XML stack via traits.
//! Also provides a method of supplying an XML stack from a file for testing purposes.

use std::collections::HashMap;
use std::io;
use std::iter::Peekable;
use std::num::ParseIntError;

use xml::reader::{EventReader, Events, ParserConfig, XmlEvent};
use xml::writer::EventWriter;

use crate::core::error::Ks3Error;
use crate::core::request::HttpResponse;

/// generic Error for XML parsing
#[derive(Debug)]
pub struct XmlParseError(pub String);

impl XmlParseError {
    pub fn new(msg: &str) -> XmlParseError {
        XmlParseError(msg.to_string())
    }
}

/// syntactic sugar for the XML event stack we pass around
pub type XmlStack<'a> = Peekable<Events<&'a [u8]>>;

/// Peek at next items in the XML stack
pub trait Peek {
    fn peek(&mut self) -> Option<&Result<XmlEvent, xml::reader::Error>>;
}

/// Move to the next part of the XML stack
pub trait Next {
    fn next(&mut self) -> Option<Result<XmlEvent, xml::reader::Error>>;
}

/// Wraps the Hyper Response type
pub struct XmlResponse<'b> {
    xml_stack: Peekable<Events<&'b [u8]>>, // refactor to use XmlStack type?
}

impl<'b> XmlResponse<'b> {
    pub fn new(stack: Peekable<Events<&'b [u8]>>) -> XmlResponse {
        XmlResponse { xml_stack: stack }
    }
}

impl<'b> Peek for XmlResponse<'b> {
    fn peek(&mut self) -> Option<&Result<XmlEvent, xml::reader::Error>> {
        while let Some(&Ok(XmlEvent::Whitespace(_))) = self.xml_stack.peek() {
            self.xml_stack.next();
        }
        self.xml_stack.peek()
    }
}

impl<'b> Next for XmlResponse<'b> {
    fn next(&mut self) -> Option<Result<XmlEvent, xml::reader::Error>> {
        let mut maybe_event;
        loop {
            maybe_event = self.xml_stack.next();
            match maybe_event {
                Some(Ok(XmlEvent::Whitespace(_))) => {}
                _ => break,
            }
        }
        maybe_event
    }
}

impl From<ParseIntError> for XmlParseError {
    fn from(_e: ParseIntError) -> XmlParseError {
        XmlParseError::new("ParseIntError")
    }
}

/// return a string field with the right name or throw a parse error
pub fn string_field<T: Peek + Next>(name: &str, stack: &mut T) -> Result<String, XmlParseError> {
    start_element(name, stack)?;
    let value = characters(stack)?;
    end_element(name, stack)?;
    Ok(value)
}

pub fn write_characters_element<W>(
    writer: &mut EventWriter<W>,
    name: &str,
    value_str: &str,
) -> Result<(), xml::writer::Error>
where
    W: io::Write,
{
    writer.write(xml::writer::XmlEvent::start_element(name))?;
    writer.write(xml::writer::XmlEvent::characters(value_str))?;
    writer.write(xml::writer::XmlEvent::end_element())
}

pub fn deserialize_primitive<T: Peek + Next, U>(
    tag_name: &str,
    stack: &mut T,
    deserialize: fn(String) -> Result<U, XmlParseError>,
) -> Result<U, XmlParseError> {
    start_element(tag_name, stack)?;
    let obj = deserialize(characters(stack)?)?;
    end_element(tag_name, stack)?;

    Ok(obj)
}

/// return some XML Characters
pub fn characters<T: Peek + Next>(stack: &mut T) -> Result<String, XmlParseError> {
    {
        // Lexical lifetime
        // Check to see if the next element is an end tag.
        // If it is, return an empty string.
        let current = stack.peek();
        if let Some(&Ok(XmlEvent::EndElement { .. })) = current {
            return Ok("".to_string());
        }
    }
    match stack.next() {
        Some(Ok(XmlEvent::Characters(data))) | Some(Ok(XmlEvent::CData(data))) => Ok(data),
        _ => Err(XmlParseError::new("Expected characters")),
    }
}

/// get the name of the current element in the stack.  throw a parse error if it's not a `StartElement`
pub fn peek_at_name<T: Peek + Next>(stack: &mut T) -> Result<String, XmlParseError> {
    let current = stack.peek();
    if let Some(&Ok(XmlEvent::StartElement { ref name, .. })) = current {
        Ok(name.local_name.to_string())
    } else {
        Ok("".to_string())
    }
}

/// consume a `StartElement` with a specific name or throw an `XmlParseError`
pub fn start_element<T: Peek + Next>(
    element_name: &str,
    stack: &mut T,
) -> Result<HashMap<String, String>, XmlParseError> {
    let next = stack.next();

    if let Some(Ok(XmlEvent::StartElement {
        name, attributes, ..
    })) = next
    {
        if name.local_name == element_name {
            let mut attr_map = HashMap::new();
            for attr in attributes {
                attr_map.insert(attr.name.local_name, attr.value);
            }
            Ok(attr_map)
        } else {
            Err(XmlParseError::new(&format!(
                "START Expected {} got {}",
                element_name, name.local_name
            )))
        }
    } else {
        Err(XmlParseError::new(&format!(
            "Expected StartElement {} got {:#?}",
            element_name, next
        )))
    }
}

/// consume an `EndElement` with a specific name or throw an `XmlParseError`
pub fn end_element<T: Peek + Next>(element_name: &str, stack: &mut T) -> Result<(), XmlParseError> {
    let next = stack.next();
    if let Some(Ok(XmlEvent::EndElement { name, .. })) = next {
        if name.local_name == element_name {
            Ok(())
        } else {
            Err(XmlParseError::new(&format!(
                "END Expected {} got {}",
                element_name, name.local_name
            )))
        }
    } else {
        Err(XmlParseError::new(&format!(
            "Expected EndElement {} got {:?}",
            element_name, next
        )))
    }
}

/// skip a tag and all its children
pub fn skip_tree<T: Peek + Next>(stack: &mut T) {
    let mut deep: usize = 0;

    loop {
        match stack.next() {
            None => break,
            Some(Ok(XmlEvent::StartElement { .. })) => deep += 1,
            Some(Ok(XmlEvent::EndElement { .. })) => {
                if deep > 1 {
                    deep -= 1;
                } else {
                    break;
                }
            }
            _ => (),
        }
    }
}

/// skip all elements until a start element is encountered
///
/// Errors and end-of-stream are ignored.
pub fn find_start_element<T: Peek + Next>(stack: &mut T) {
    loop {
        match stack.peek() {
            Some(&Ok(XmlEvent::StartElement { .. })) => break,
            Some(&Ok(_)) => {
                stack.next().unwrap().unwrap();
            }
            Some(&Err(_)) => break,
            None => break,
        }
    }
}

pub fn deserialize_elements<T, S, F>(
    tag_name: &str,
    stack: &mut T,
    mut handle_element: F,
) -> Result<S, XmlParseError>
where
    T: Peek + Next,
    S: Default,
    F: FnMut(&str, &mut T, &mut S) -> Result<(), XmlParseError>,
{
    let mut obj = S::default();

    start_element(tag_name, stack)?;

    loop {
        match stack.peek() {
            Some(&Ok(XmlEvent::EndElement { .. })) => break,
            Some(&Ok(XmlEvent::StartElement { ref name, .. })) => {
                let local_name = name.local_name.to_owned();
                handle_element(&local_name, stack, &mut obj)?;
            }
            _ => {
                stack.next();
            }
        }
    }

    end_element(tag_name, stack)?;

    Ok(obj)
}

pub async fn parse_response<T, E>(
    response: &mut HttpResponse,
    deserialize: fn(&str, &mut XmlResponse<'_>) -> Result<T, XmlParseError>,
) -> Result<T, Ks3Error<E>>
where
    T: Default,
{
    let xml_response = response.buffer().await.map_err(Ks3Error::HttpDispatch)?;
    if xml_response.body.is_empty() {
        Ok(T::default())
    } else {
        let reader = EventReader::new_with_config(
            xml_response.body.as_ref(),
            ParserConfig::new().trim_whitespace(false),
        );
        let mut stack = XmlResponse::new(reader.into_iter().peekable());
        let _start_document = stack.next();
        let actual_tag_name = peek_at_name(&mut stack)?;
        Ok(deserialize(&actual_tag_name, &mut stack)?)
    }
}
