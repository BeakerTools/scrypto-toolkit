use crate::internal_prelude::*;

pub trait ToId {
    fn to_id(self) -> NonFungibleLocalId;
}

impl ToId for NonFungibleLocalId {
    fn to_id(self) -> NonFungibleLocalId {
        self
    }
}

impl ToId for u64 {
    fn to_id(self) -> NonFungibleLocalId {
        NonFungibleLocalId::integer(self)
    }
}

impl ToId for Vec<u8> {
    fn to_id(self) -> NonFungibleLocalId {
        NonFungibleLocalId::bytes(self).unwrap()
    }
}

impl ToId for [u8; 32] {
    fn to_id(self) -> NonFungibleLocalId {
        NonFungibleLocalId::ruid(self)
    }
}

impl ToId for String {
    fn to_id(self) -> NonFungibleLocalId {
        // Transform the String into a vec of chars
        let mut chars: Vec<char> = self.chars().collect();

        // Check if it starts by '<', '#', '{' or '['
        if let Some(first) = chars.first() {
            match first {
                '<' => {
                    chars.remove(0);
                    let last_char = chars.pop().unwrap();
                    if last_char == '>' && !chars.is_empty() {
                        // Then it is an integer nf id and we skip at the end
                    } else {
                        chars.push(last_char);
                    }
                }
                '#' => {
                    chars.remove(0);
                    let last_char = chars.pop().unwrap();
                    if last_char == '#' && !chars.is_empty() {
                        // Then it is an integer nf id

                        let char_str: String = chars.into_iter().collect();
                        let id: u64 = char_str.parse().unwrap();
                        return id.to_id();
                    } else {
                        chars.push(last_char);
                    }
                }
                '[' => {
                    chars.remove(0);
                    let last_char = chars.pop().unwrap();
                    if last_char == ']' && !chars.is_empty() {
                        // Then it is a bytes nf id

                        let hex_string: String = chars.into_iter().collect();
                        let hex = hex_string.as_str();
                        let bytes: Vec<u8> = (0..hex.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                            .collect();
                        return bytes.to_id();
                    } else {
                        chars.push(last_char);
                    }
                }
                '{' => {
                    chars.remove(0);
                    let last_char = chars.pop().unwrap();
                    if last_char == '}' && !chars.is_empty() {
                        // Then it is a ruid nf id

                        let hex_string: String =
                            chars.into_iter().collect::<String>().replace('-', "");
                        let hex_cleaned = hex_string.as_str();
                        let mut bytes = [0u8; 32];
                        for i in 0..32 {
                            bytes[i] =
                                u8::from_str_radix(&hex_cleaned[i * 2..i * 2 + 2], 16).unwrap();
                        }

                        return bytes.to_id();
                    } else {
                        chars.push(last_char);
                    }
                }
                _ => { // Else do nothing }
                }
            }
        }
        // In other cases we consider that it is a String non-fungible id
        let char_str: String = chars.into_iter().collect();
        NonFungibleLocalId::string(char_str).unwrap()
    }
}

macro_rules! integer_types_impl {
    ($type_name: ident) => {
        impl ToId for $type_name {
            fn to_id(self) -> NonFungibleLocalId {
                NonFungibleLocalId::integer(u64::try_from(self).unwrap())
            }
        }
    };
}

integer_types_impl!(u8);
integer_types_impl!(u16);
integer_types_impl!(u32);
integer_types_impl!(u128);
integer_types_impl!(i8);
integer_types_impl!(i16);
integer_types_impl!(i32);
integer_types_impl!(i64);
integer_types_impl!(i128);

macro_rules! subtypes_impl {
    ($type_name: ident, $subtype_name: ident) => {
        impl ToId for $type_name {
            fn to_id(self) -> NonFungibleLocalId {
                NonFungibleLocalId::$subtype_name(self)
            }
        }
    };
}

subtypes_impl!(BytesNonFungibleLocalId, Bytes);
subtypes_impl!(IntegerNonFungibleLocalId, Integer);
subtypes_impl!(RUIDNonFungibleLocalId, RUID);
subtypes_impl!(StringNonFungibleLocalId, String);
