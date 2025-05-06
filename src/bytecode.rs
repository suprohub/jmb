use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use bitvec::{prelude::*, view::BitView};
use serde::{
    Serialize, Serializer,
    ser::{
        self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
};

use crate::module::Module;

pub fn compile(module: Module) -> String {
    let mut jmb = Jmb::new();
    module.serialize(&mut jmb).unwrap();
    jmb.finalize();
    String::from_utf8_lossy(&jmb.bits.into_vec()).to_string()
}

pub fn decompile() {
    // todo
}

pub struct Jmb {
    bits: BitVec<u8, Lsb0>,
    strs: HashMap<String, Vec<usize>>,
}

impl Jmb {
    pub fn new() -> Self {
        Self {
            bits: BitVec::new(),
            strs: HashMap::new(),
        }
    }

    pub fn finalize(&mut self) {
        if self.strs.is_empty() {
            return;
        }

        // Собираем уникальные строки и сортируем их для детерминированного порядка
        let mut strings: Vec<&String> = self.strs.keys().collect();
        strings.sort();

        // Создаем карту для быстрого поиска индекса строки
        let string_indices: HashMap<&String, usize> =
            strings.iter().enumerate().map(|(i, s)| (*s, i)).collect();

        // Вычисляем количество бит на индекс
        let num_strings = strings.len();
        let bits_per_index = if num_strings <= 1 {
            0
        } else {
            (num_strings - 1).ilog2() + 1
        };

        // Собираем все позиции, которые нужно обновить, с их индексами
        let mut all_positions: Vec<(usize, usize)> = Vec::new();
        for (s, positions) in &self.strs {
            let index = string_indices[s];
            for &pos in positions {
                all_positions.push((pos, index));
            }
        }
        // Сортируем позиции по возрастанию
        all_positions.sort_by_key(|&(pos, _)| pos);

        // Создаем временный битовый вектор для строк
        let mut string_bits = BitVec::new();
        for s in strings {
            let len = s.len() as u16;
            // Добавляем длину строки (16 бит)
            string_bits.extend_from_bitslice(&len.view_bits::<Lsb0>()[..16]);
            // Добавляем байты строки (по 8 бит каждый)
            for &byte in s.as_bytes() {
                string_bits.extend_from_bitslice(&byte.view_bits::<Lsb0>()[..8]);
            }
        }

        // Создаем новый битовый вектор, начиная со строк
        let mut new_bits = string_bits;

        // Добавляем исходные биты, заменяя позиции на индексы строк
        let mut current_pos = 0;
        let mut temp_original_bits = self.bits.clone();
        for (pos, index) in all_positions {
            // Проверяем, что позиция находится в пределах исходных битов
            if pos > temp_original_bits.len() {
                temp_original_bits.resize(pos, false);
            }

            // Добавляем биты до текущей позиции
            new_bits.extend_from_bitslice(&temp_original_bits[current_pos..pos]);

            // Добавляем индекс строки (обрезаем до bits_per_index бит)
            let index_bits = &index.view_bits::<Lsb0>()[..bits_per_index as usize];
            new_bits.extend_from_bitslice(index_bits);

            current_pos = pos;
        }

        // Добавляем оставшиеся биты после последней позиции
        new_bits.extend_from_bitslice(&temp_original_bits[current_pos..]);

        // Заменяем старый битовый вектор новым
        self.bits = new_bits;
        self.strs.clear();
    }
}

macro_rules! other_side {
    (i8) => {
        u8
    };
    (i16) => {
        u16
    };
    (i32) => {
        u32
    };
    (i64) => {
        u64
    };
    (i128) => {
        u128
    };
    (isize) => {
        usize
    };
    (f32) => {
        u32
    };
    (f64) => {
        u64
    };
    ($other:ty) => {
        $other
    };
}

macro_rules! auto_impl {
    ($($t:ty),* => $v1:ident, $v2:ident $body:block) => {
        $(
            paste::paste! {
                fn [<serialize_ $t>]($v1, $v2: $t) -> Result<Self::Ok, Self::Error> {
                    $body
                }
            }
        )*
    };
    ($($t:ty),* => where $v1:ident, $v2:ident $body:block) => {
        $(
            paste::paste! {
                fn [<serialize_ $t>]($v1, $v2: $t) -> Result<Self::Ok, Self::Error> {
                    type OtherSide = other_side!($t);
                    $body
                }
            }
        )*
    };
}

impl Serializer for &mut Jmb {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Err;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing {v}");
        self.bits.push(v);
        Ok(())
    }

    auto_impl!(i8, i16, i32, i64 => where self, v {
        log::info!("Serializing signed int {v}");
        self.bits.extend_from_bitslice((v as OtherSide).view_bits::<Lsb0>());
        Ok(())
    });

    auto_impl!(u8, u16, u32, u64 => self, v {
        log::info!("Serializing unsigned int {v}");
        self.bits.extend_from_bitslice(v.view_bits::<Lsb0>());
        Ok(())
    });

    auto_impl!(f32, f64 => where self, v {
        log::info!("Serializing float {v}");

        // SAFETY: We just swiched to unsigned with same bytes and byte order
        // Nothing special or breaking
        let v: OtherSide = unsafe { std::mem::transmute(v) };
        log::info!("Transmuted to: {v}");
        self.bits.extend_from_bitslice(v.view_bits::<Lsb0>());
        Ok(())
    });

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing char {v:?}");
        self.bits
            .extend_from_bitslice((v as u32).view_bits::<Lsb0>());
        Ok(())
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing str {v:?}");
        let len = self.bits.len();

        self.strs
            .entry(v.to_string())
            .and_modify(|vec| vec.push(len))
            .or_default();

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing bytes {v:?}");
        self.bits.extend_from_bitslice(v.as_bits::<Lsb0>());
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing none..");
        Ok(())
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        log::info!("Serializing some");
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing unit..");
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing unit struct..");
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        log::info!("Serializing unit variant {name}::{variant} as {variant_index}");

        if name.ends_with("Bits") && name.contains("Wants") {
            let bits: usize = name
                .split_once("Wants")
                .unwrap()
                .1
                .trim_end_matches("Bits")
                .parse()
                .unwrap();

            log::info!("Variant wants {bits} bits");

            self.bits
                .extend_from_bitslice(&variant_index.view_bits::<Lsb0>()[..bits]);
        } else {
            self.bits
                .extend_from_bitslice(variant_index.view_bits::<Lsb0>());
        }
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        log::info!("Serializing newtype struct {name}");
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        log::info!("Serializing newtype variant {name}::{variant} as {variant_index}");

        if name.ends_with("Bits") && name.contains("Wants") {
            let bits: usize = name
                .split_once("Wants")
                .unwrap()
                .1
                .trim_end_matches("Bits")
                .parse()
                .unwrap();

            log::info!("Variant wants {bits} bits");

            self.bits
                .extend_from_bitslice(&variant_index.view_bits::<Lsb0>()[..bits]);
        } else {
            self.bits
                .extend_from_bitslice(variant_index.view_bits::<Lsb0>());
        }

        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        log::info!("Serializing seq");
        self.bits
            .extend_from_bitslice(len.unwrap().view_bits::<Lsb0>());
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.bits.extend_from_bitslice(len.view_bits::<Lsb0>());

        if name.ends_with("Bits") && name.contains("Wants") {
            let bits: usize = name
                .split_once("Wants")
                .unwrap()
                .1
                .trim_end_matches("Bits")
                .parse()
                .unwrap();

            log::info!("Variant wants {bits} bits");

            self.bits
                .extend_from_bitslice(&variant_index.view_bits::<Lsb0>()[..bits]);
        } else {
            self.bits
                .extend_from_bitslice(variant_index.view_bits::<Lsb0>());
        }

        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        log::info!("Serializing map");
        self.bits
            .extend_from_bitslice(len.unwrap().view_bits::<Lsb0>());
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        log::info!("Serializing struct");
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        log::info!("Serializing struct variant");

        if name.ends_with("Bits") && name.contains("Wants") {
            let bits: usize = name
                .split_once("Wants")
                .unwrap()
                .1
                .trim_end_matches("Bits")
                .parse()
                .unwrap();

            log::info!("Variant wants {bits} bits");

            self.bits
                .extend_from_bitslice(&variant_index.view_bits::<Lsb0>()[..bits]);
        } else {
            self.bits
                .extend_from_bitslice(variant_index.view_bits::<Lsb0>());
        }

        Ok(self)
    }
}

impl SerializeSeq for &mut Jmb {
    type Ok = ();
    type Error = Err;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTuple for &mut Jmb {
    type Ok = ();
    type Error = Err;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleStruct for &mut Jmb {
    type Ok = ();
    type Error = Err;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTupleVariant for &mut Jmb {
    type Ok = ();
    type Error = Err;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeMap for &mut Jmb {
    type Ok = ();
    type Error = Err;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStruct for &mut Jmb {
    type Ok = ();
    type Error = Err;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStructVariant for &mut Jmb {
    type Ok = ();
    type Error = Err;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum Err {
    A,
}

impl Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl ser::Error for Err {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::A
    }
}

impl std::error::Error for Err {}

pub fn as_2bits<S: Serializer, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize + BitStore,
    <T as BitStore>::Mem: Serialize,
{
    value.view_bits::<Lsb0>()[..2].serialize(serializer)
}
