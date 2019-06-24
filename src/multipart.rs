//! Serialize a value into a request multipart `Form`.

use std::borrow::Cow;
use serde::ser::{
    Error as SerError,
    Serialize,
    Serializer,
    SerializeSeq,
    SerializeTuple,
    SerializeTupleStruct,
    SerializeTupleVariant,
    SerializeMap,
    SerializeStruct,
    SerializeStructVariant,
};
use reqwest::multipart::{ Form, Part };
use crate::error::{ Error, Result };

/// Describes a file in a multipart form.
/// The first string is the contents of the file, the second is the
/// file name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "$FormFile")]
pub struct FormFile(String, String);

impl FormFile {
    /// The constructor. First parameter is the file contents, the second is
    /// the file name.
    pub fn with_contents_and_file_name<T, U>(contents: T, file_name: U) -> Self
        where T: Into<String>,
              U: Into<String>,
    {
        FormFile(contents.into(), file_name.into())
    }

    /// Returns the contents, discarding the file name.
    pub fn into_contents(self) -> String {
        self.0
    }

    /// Returns the file name, discarding the contents.
    pub fn into_file_name(self) -> String {
        self.1
    }

    /// Returns a reference to the contents.
    pub fn contents(&self) -> &str {
        &self.0
    }

    /// Returns a reference to the file name.
    pub fn file_name(&self) -> &str {
        &self.1
    }

    /// Replaces the contents of this file part with the given argument.
    pub fn set_contents(&mut self, contents: String) {
        self.0 = contents;
    }

    /// Replaces the file name of this file part with the given argument.
    pub fn set_file_name(&mut self, file_name: String) {
        self.1 = file_name;
    }
}

/// Takes a serializable value and turns into a multipart form.
pub fn to_form<T: Serialize>(value: &T) -> Result<Form> {
    let mut serializer = FormSerializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.form.expect("form should never be None"))
}

/// Serializer for encoding values as multipart/form-data.
#[derive(Debug)]
struct FormSerializer {
    /// Are we currently serializing the top-level map or struct?
    serializing_map: bool,
    /// Are we currently serializing a file part?
    serializing_file: bool,
    /// The current key when we are serializing a struct.
    current_key: Option<Cow<'static, str>>,
    /// The current file contents when we are serializing a file part.
    current_file_contents: Option<Cow<'static, str>>,
    /// The result being built.
    form: Option<Form>,
}

impl Default for FormSerializer {
    fn default() -> Self {
        FormSerializer {
            serializing_map: false,
            serializing_file: false,
            current_key: None,
            current_file_contents: None,
            form: Some(Form::new()),
        }
    }
}

impl FormSerializer {
    /// Serialize a string as either a map key or the corresponding value.
    fn serialize_form_string<T>(&mut self, string: T) -> Result<()>
        where T: Into<Cow<'static, str>>
    {
        if self.serializing_map {
            let string = string.into();

            // If a key already exists, we are a value, otherwise we are a key.
            match self.current_key.take() {
                Some(key) => {
                    if self.serializing_file {
                        match self.current_file_contents.take() {
                            Some(contents) => {
                                let form = self.form.take().expect("form should never be None");
                                let part = Part::text(contents).file_name(string);
                                self.form.replace(form.part(key, part));
                            }
                            None => {
                                self.current_file_contents.replace(string);
                                self.current_key.replace(key); // put it back
                            }
                        }
                    } else {
                        let form = self.form.take().expect("form should never be None");
                        self.form.replace(form.text(key, string));
                    }
                }
                None => self.current_key = Some(string),
            }

            Ok(())
        } else {
            Err(Error::custom(
                "top-level value to be serialized as multipart should be a map or a struct"
            ))
        }
    }

    /// Serialize a binary blob as the value corresponding to the current key.
    fn serialize_form_blob<T>(&mut self, blob: T) -> Result<()>
        where T: Into<Cow<'static, [u8]>>
    {
        if self.serializing_map {
            // If a key already exists, we are a value, otherwise we are a key.
            self.current_key.take().map_or_else(
                || Err(Error::custom(
                    "binary blob can't be serialized as a key, only as a value"
                )),
                |key| {
                    let form = self.form.take().expect("form should never be None");
                    self.form.replace(form.part(key, Part::bytes(blob)));
                    Ok(())
                }
            )
        } else {
            Err(Error::custom(
                "top-level value to be serialized as multipart should be a map or a struct"
            ))
        }
    }
}

impl Serializer for &mut FormSerializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_form_string(if v { "true" } else { "false" })
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.serialize_form_string(v.to_string())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_form_string(v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.serialize_form_string(v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_form_string(v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.serialize_form_string(v.to_owned())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.serialize_form_blob(v.to_owned())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.serialize_form_string("null")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        let mut map = self.serialize_map(Some(1))?;
        map.serialize_entry(variant, value)?;
        SerializeMap::end(map)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::custom(
            "can't serialize a sequence as multipart"
        ))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        if !self.serializing_map {
            return Err(Error::custom("can't serialize tuple struct at top level"));
        }

        if self.current_key.is_none() {
            return Err(Error::custom("can't serialize tuple struct as field name"));
        }

        if name != "$FormFile" || len != 2 {
            return Err(Error::custom("only form file can be serialized as tuple struct"));
        }

        if self.serializing_file {
            return Err(Error::custom("can't serialize a tuple struct inside a file"));
        }

        self.serializing_file = true;

        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _enum_name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::custom(
            "can't serialize a tuple variant as multipart"
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        if self.serializing_map {
            Err(Error::custom(
                "can't serialize a map-like value as multipart unless it's at the top level"
            ))
        } else {
            self.serializing_map = true;
            Ok(self)
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::custom(
            "can't serialize a struct variant as multipart"
        ))
    }
}

impl<'a> SerializeSeq for &'a mut FormSerializer {
    type Ok = <&'a mut FormSerializer as Serializer>::Ok;
    type Error = <&'a mut FormSerializer as Serializer>::Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a sequence as multipart"
        ))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a sequence as multipart"
        ))
    }
}

impl<'a> SerializeTuple for &'a mut FormSerializer {
    type Ok = <&'a mut FormSerializer as Serializer>::Ok;
    type Error = <&'a mut FormSerializer as Serializer>::Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a tuple as multipart"
        ))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a tuple as multipart"
        ))
    }
}

impl<'a> SerializeTupleStruct for &'a mut FormSerializer {
    type Ok = <&'a mut FormSerializer as Serializer>::Ok;
    type Error = <&'a mut FormSerializer as Serializer>::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        if self.serializing_file {
            value.serialize(&mut **self)
        } else {
            Err(Error::custom(
                "can't serialize a tuple struct as multipart"
            ))
        }
    }

    fn end(self) -> Result<Self::Ok> {
        assert!(self.serializing_map);
        assert!(self.serializing_file);

        if self.current_key.is_some() {
            Err(Error::custom("missing contents from file form part"))
        } else if self.current_file_contents.is_some() {
            Err(Error::custom("missing file_name from file form part"))
        } else {
            self.serializing_file = false;
            Ok(())
        }
    }
}

impl<'a> SerializeTupleVariant for &'a mut FormSerializer {
    type Ok = <&'a mut FormSerializer as Serializer>::Ok;
    type Error = <&'a mut FormSerializer as Serializer>::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a tuple variant as multipart"
        ))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a tuple variant as multipart"
        ))
    }
}

impl<'a> SerializeStructVariant for &'a mut FormSerializer {
    type Ok = <&'a mut FormSerializer as Serializer>::Ok;
    type Error = <&'a mut FormSerializer as Serializer>::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, _value: &T) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a struct variant as multipart"
        ))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::custom(
            "can't serialize a struct variant as multipart"
        ))
    }
}

impl<'a> SerializeMap for &'a mut FormSerializer {
    type Ok = <&'a mut FormSerializer as Serializer>::Ok;
    type Error = <&'a mut FormSerializer as Serializer>::Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<Self::Ok> {
        if self.current_key.is_none() {
            key.serialize(&mut **self)
        } else {
            Err(Error::custom(
                "can't serialize two keys in a row without a value"
            ))
        }
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        if self.current_key.is_some() {
            value.serialize(&mut **self)
        } else {
            Err(Error::custom(
                "can't serialize value without first serializing a key"
            ))
        }
    }

    fn end(self) -> Result<Self::Ok> {
        if self.current_key.is_none() {
            Ok(())
        } else {
            Err(Error::custom("missing value after key"))
        }
    }
}

impl<'a> SerializeStruct for &'a mut FormSerializer {
    type Ok = <&'a mut FormSerializer as Serializer>::Ok;
    type Error = <&'a mut FormSerializer as Serializer>::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<Self::Ok> {
        self.serialize_entry(key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        SerializeMap::end(self)
    }
}
