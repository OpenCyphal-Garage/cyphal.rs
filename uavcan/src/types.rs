pub use ux::{
    u2, u3, u4, u5, u6, u7, u9, u10, u11, u12, u13, u14, u15, u17, u18, u19, u20, u21, u22, u23, u24, u25, u26, u27, u28, u29, u30, u31,
    u33, u34, u35, u36, u37, u38, u39, u40, u41, u42, u43, u44, u45, u46, u47, u48, u49, u50, u51, u52, u53, u54, u55, u56, u57, u58, u59, u60, u61, u62, u63,
    i2, i3, i4, i5, i6, i7, i9, i10, i11, i12, i13, i14, i15, i17, i18, i19, i20, i21, i22, i23, i24, i25, i26, i27, i28, i29, i30, i31,
    i33, i34, i35, i36, i37, i38, i39, i40, i41, i42, i43, i44, i45, i46, i47, i48, i49, i50, i51, i52, i53, i54, i55, i56, i57, i58, i59, i60, i61, i62, i63,
};
pub use half::f16;

use lib;
use lib::core::fmt;
use lib::core::cmp;
use lib::core::ops::{
    Index,
    IndexMut,
};

use bit_field::BitField;

use serializer::{
    SerializationResult,
    SerializationBuffer,
};

use deserializer::{
    DeserializationResult,
    DeserializationBuffer,
};

/// This trait is only exposed so `Struct` can be derived.
/// It is not intended for use outside the derive macro and
/// must not be considered as a stable part of the API.
#[doc(hidden)]
pub trait PrimitiveType : Sized + Copy{
    const BIT_LENGTH: usize;
    
    /// Mask bits exceeding `BIT_LENGTH`
    fn from_bits(v: u64) -> Self;
    
    /// Zeroes bits exceeding `BIT_LENGTH`
    fn to_bits(self) -> u64;

    fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult {            
        let type_bits_remaining = Self::BIT_LENGTH - *bit;
        let buffer_bits_remaining = buffer.bits_remaining();
        
        if type_bits_remaining == 0 {
            SerializationResult::Finished
        } else if buffer_bits_remaining == 0 {
            SerializationResult::BufferFull
        } else if buffer_bits_remaining >= type_bits_remaining {
            buffer.push_bits(type_bits_remaining, (self.to_bits() >> *bit));
            *bit = Self::BIT_LENGTH;
            SerializationResult::Finished
        } else {
            buffer.push_bits(buffer_bits_remaining, (self.to_bits() >> *bit));
            *bit += buffer_bits_remaining;
            SerializationResult::BufferFull
        }
    }

    fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
        let buffer_len = buffer.bit_length();
        if buffer_len == 0 && *bit == Self::BIT_LENGTH {
            DeserializationResult::Finished
        } else if buffer_len == 0 && *bit != Self::BIT_LENGTH {
            DeserializationResult::BufferInsufficient
        } else if buffer_len < Self::BIT_LENGTH - *bit {
            *self = Self::from_bits(self.to_bits() | (buffer.pop_bits(buffer_len) << *bit));
            *bit += buffer_len;
            DeserializationResult::BufferInsufficient
        } else {
            *self = Self::from_bits(self.to_bits() | (buffer.pop_bits(Self::BIT_LENGTH-*bit) << *bit));
            *bit += Self::BIT_LENGTH;
            DeserializationResult::Finished
        }
        
    }
}

/// This trait is only exposed so `Struct` can be derived.
/// It is not intended for use outside the derive macro and
/// must not be considered as a stable part of the API.
#[doc(hidden)]
pub trait Array {
    const LENGTH: usize;
    const BIT_LENGTH: usize;
    type ELEMENT_TYPE;
    
    fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult;
    fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;
}

macro_rules! impl_array{
    {[$($size:expr), *]} => {$(impl_array!($size);)*};
    {$size:expr} => {
        impl<T: PrimitiveType> Array for [T; $size] {
            const LENGTH: usize = $size;
            const BIT_LENGTH: usize = $size * T::BIT_LENGTH;
            type ELEMENT_TYPE = T;
            
            fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult {
                let mut start_element = *bit / T::BIT_LENGTH;
                let start_element_bit = *bit % T::BIT_LENGTH;
                
                // first get rid of the odd bits
                if start_element_bit != 0 {
                    let mut bits_serialized = start_element_bit;
                    match self[start_element].serialize(&mut bits_serialized, buffer) {
                        SerializationResult::Finished => {
                            *bit += bits_serialized;
                        },
                        SerializationResult::BufferFull => {
                            *bit += bits_serialized;
                            return SerializationResult::BufferFull;
                        },
                    }
                    start_element += 1;
                }
                
                for element in self.iter().skip(start_element) {
                    let mut bits_serialized = 0;
                    match element.serialize(&mut bits_serialized, buffer) {
                        SerializationResult::Finished => {
                            *bit += bits_serialized;
                        },
                        SerializationResult::BufferFull => {
                            *bit += bits_serialized;
                            return SerializationResult::BufferFull;
                        },
                    }
                }
                
                SerializationResult::Finished
            }
            
            fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
                
                let mut start_element = *bit / T::BIT_LENGTH;
                let start_element_bit = *bit % T::BIT_LENGTH;
                
                // first get rid of the odd bits
                if start_element_bit != 0 {
                    let mut bits_deserialized = start_element_bit;
                    match self[start_element].deserialize(&mut bits_deserialized, buffer) {
                        DeserializationResult::Finished => {
                            *bit += bits_deserialized;
                        },
                        DeserializationResult::BufferInsufficient => {
                            *bit += bits_deserialized;
                            return DeserializationResult::BufferInsufficient;
                        },
                    }
                    start_element += 1;
                }
                
                for element in self.iter_mut().skip(start_element) {
                    let mut bits_deserialized = start_element_bit;
                    match element.deserialize(&mut bits_deserialized, buffer) {
                        DeserializationResult::Finished => {
                            *bit += bits_deserialized;
                        },
                        DeserializationResult::BufferInsufficient => {
                            *bit += bits_deserialized;
                            return DeserializationResult::BufferInsufficient;
                        },
                    }
                }

                DeserializationResult::Finished
            }
        }
    };
}

impl_array!([1, 2, 3, 4, 5, 6, 7, 8, 9]);
impl_array!([10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
impl_array!([20, 21, 22, 23, 24, 25, 26, 27, 28, 29]);
impl_array!([30, 31, 32, 33, 34, 35, 36, 37, 38, 39]);
impl_array!([40, 41, 42, 43, 44, 45, 46, 47, 48, 49]);
impl_array!([50, 51, 52, 53, 54, 55, 56, 57, 58, 59]);
impl_array!([60, 61, 62, 63, 64, 65, 66, 67, 68, 69]);
impl_array!([70, 71, 72, 73, 74, 75, 76, 77, 78, 79]);
impl_array!([80, 81, 82, 83, 84, 85, 86, 87, 88, 89]);
impl_array!([90, 91, 92, 93, 94, 95, 96, 97, 98, 99]);

impl_array!([100, 101, 102, 103, 104, 105, 106, 107, 108, 109]);
impl_array!([110, 111, 112, 113, 114, 115, 116, 117, 118, 119]);
impl_array!([120, 121, 122, 123, 124, 125, 126, 127, 128, 129]);
impl_array!([130, 131, 132, 133, 134, 135, 136, 137, 138, 139]);
impl_array!([140, 141, 142, 143, 144, 145, 146, 147, 148, 149]);
impl_array!([150, 151, 152, 153, 154, 155, 156, 157, 158, 159]);
impl_array!([160, 161, 162, 163, 164, 165, 166, 167, 168, 169]);
impl_array!([170, 171, 172, 173, 174, 175, 176, 177, 178, 179]);
impl_array!([180, 181, 182, 183, 184, 185, 186, 187, 188, 189]);
impl_array!([190, 191, 192, 193, 194, 195, 196, 197, 198, 199]);

impl_array!([200, 201, 202, 203, 204, 205, 206, 207, 208, 209]);
impl_array!([210, 211, 212, 213, 214, 215, 216, 217, 218, 219]);
impl_array!([220, 221, 222, 223, 224, 225, 226, 227, 228, 229]);
impl_array!([230, 231, 232, 233, 234, 235, 236, 237, 238, 239]);
impl_array!([240, 241, 242, 243, 244, 245, 246, 247, 248, 249]);
impl_array!([250, 251, 252, 253, 254, 255]);

/// The Uavcan dynamic array type
///
/// # Examples
/// ```
/// use uavcan::types::*;
///
/// let dynamic_array = Dynamic::<[u8; 90]>::with_data("dynamic array".as_bytes());
///
/// assert_eq!(dynamic_array.length(), 13);
///
/// ```
pub struct Dynamic<T> {
    array: T,
    current_length: usize,
}

macro_rules! impl_dynamic{
    {[$(($size:expr, $length_bits:expr)), *]} => {$(impl_dynamic!(($size, $length_bits));)*};
    {($size:expr, $length_bits:expr)} => {
        impl<T: PrimitiveType> Dynamic<[T; $size]> {
            pub const LENGTH_BITS: usize = $length_bits;
            pub const MAX_LENGTH: usize = $size;

            pub fn with_data(data: &[T]) -> Self {
                let mut s = Self{
                    array: [data[0]; $size],
                    current_length: data.len(),
                };
                s.array[0..data.len()].clone_from_slice(data);
                s
            }

            pub fn length(&self) -> usize {
                self.current_length
            }

            pub fn set_length(&mut self, length: usize) {
                self.current_length = length;
            }

            pub fn iter(&self) -> lib::core::slice::Iter<T> {
                self.array[0..self.current_length].iter()
            }
            
            pub fn iter_mut(&mut self) -> lib::core::slice::IterMut<T> {
                self.array[0..self.current_length].iter_mut()
            }

            /// This method is only exposed so `Struct` can be derived.
            /// It is not intended for use outside the derive macro and
            /// must not be considered as a stable part of the API.
            #[doc(hidden)]
            pub fn serialize(&self, bit: &mut usize, buffer: &mut SerializationBuffer) -> SerializationResult {
                // serialize length
                if *bit < Self::LENGTH_BITS {

                    let type_bits_remaining = Self::LENGTH_BITS - *bit;
                    let buffer_bits_remaining = buffer.bits_remaining();
                    
                    if buffer_bits_remaining >= type_bits_remaining {
                        buffer.push_bits(type_bits_remaining, self.current_length.get_bits((*bit as u8)..(Self::LENGTH_BITS as u8)) as u64);
                        *bit = Self::LENGTH_BITS;
                    } else {
                        buffer.push_bits(buffer_bits_remaining, self.current_length.get_bits((*bit as u8)..(*bit + buffer_bits_remaining) as u8) as u64);
                        *bit += buffer_bits_remaining;
                        return SerializationResult::BufferFull
                    }
                }

                let mut start_element = (*bit - Self::LENGTH_BITS) / T::BIT_LENGTH;
                let start_element_bit = (*bit - Self::LENGTH_BITS) % T::BIT_LENGTH;
                
                // first get rid of the odd bits
                if start_element_bit != 0 {
                    let mut bits_serialized = start_element_bit;
                    match self[start_element].serialize(&mut bits_serialized, buffer) {
                        SerializationResult::Finished => {
                            *bit += bits_serialized;
                        },
                        SerializationResult::BufferFull => {
                            *bit += bits_serialized;
                            return SerializationResult::BufferFull;
                        },
                    }
                    start_element += 1;
                }
                
                for element in self.iter().skip(start_element) {
                    let mut bits_serialized = 0;
                    match element.serialize(&mut bits_serialized, buffer) {
                        SerializationResult::Finished => {
                            *bit += bits_serialized;
                        },
                        SerializationResult::BufferFull => {
                            *bit += bits_serialized;
                            return SerializationResult::BufferFull;
                        },
                    }
                }
                
                SerializationResult::Finished
            }

            /// This method is only exposed so `Struct` can be derived.
            /// It is not intended for use outside the derive macro and
            /// must not be considered as a stable part of the API.
            #[doc(hidden)]
            pub fn deserialize(&mut self, bit: &mut usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
                // deserialize length
                if *bit < Self::LENGTH_BITS {
                    
                    let buffer_len = buffer.bit_length();
                    if buffer_len + *bit < Self::LENGTH_BITS {
                        self.current_length.set_bits(*bit as u8..(*bit+buffer_len) as u8, buffer.pop_bits(buffer_len) as usize);
                        *bit += buffer_len;
                        return DeserializationResult::BufferInsufficient
                    } else {
                        self.current_length.set_bits(*bit as u8..Self::LENGTH_BITS as u8, buffer.pop_bits(Self::LENGTH_BITS-*bit) as usize);
                        *bit = Self::LENGTH_BITS;
                    }
                }

                let mut start_element = (*bit - Self::LENGTH_BITS) / T::BIT_LENGTH;
                let start_element_bit = (*bit - Self::LENGTH_BITS) % T::BIT_LENGTH;

                // first get rid of the odd bits
                if start_element_bit != 0 {
                    let mut bits_deserialized = start_element_bit;
                    match self[start_element].deserialize(&mut bits_deserialized, buffer) {
                        DeserializationResult::Finished => {
                            *bit += bits_deserialized;
                        },
                        DeserializationResult::BufferInsufficient => {
                            *bit += bits_deserialized;
                            return DeserializationResult::BufferInsufficient;
                        },
                    }
                    start_element += 1;
                }
                
                for element in self.iter_mut().skip(start_element) {
                    let mut bits_deserialized = start_element_bit;
                    match element.deserialize(&mut bits_deserialized, buffer) {
                        DeserializationResult::Finished => {
                            *bit += bits_deserialized;
                        },
                        DeserializationResult::BufferInsufficient => {
                            *bit += bits_deserialized;
                            return DeserializationResult::BufferInsufficient;
                        },
                    }
                }

                DeserializationResult::Finished
            }
            
        }

        impl<T: PrimitiveType> Index<usize> for Dynamic<[T; $size]> {
            type Output = T;
            
            fn index(&self, index: usize) -> &T {
                &self.array[0..self.current_length][index]
            }
        }
        
        impl< T: PrimitiveType> IndexMut<usize> for Dynamic<[T; $size]> {
            fn index_mut(&mut self, index: usize) -> &mut T {
                &mut self.array[0..self.current_length][index]
            }
        }

        impl<T> AsRef<[T]> for Dynamic<[T; $size]> {
            fn as_ref(&self) -> &[T] {
                &self.array[0..self.current_length]
            }
        }

        impl<T> AsMut<[T]> for Dynamic<[T; $size]> {
            fn as_mut(&mut self) -> &mut [T] {
                &mut self.array[0..self.current_length]
            }
        }

        // This is needed since it can't be derived for arrays larger than 32 yet
        impl<T: PrimitiveType + cmp::PartialEq> cmp::PartialEq for Dynamic<[T; $size]> {
            fn eq(&self, other: &Self) -> bool {
                if self.current_length != other.current_length {
                    return false;
                }
                
                for i in 0..self.current_length {
                    if self.array[i] != other.array[i] {
                        return false;
                    }
                }

                true
            }
        }
        
        // This is needed since it can't be derived for arrays larger than 32 yet
        impl<T: PrimitiveType + fmt::Debug> fmt::Debug for Dynamic<[T; $size]> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "$i<T> {{ data: [")?;
                for i in 0..self.current_length {
                    write!(f, "{:?}, ", self.array[i])?;
                }
                write!(f, "]}}")
            }
        }
        
    };
}

impl_dynamic!([(1, 1), (2, 2), (3, 2), (4, 3), (5, 3), (6, 3), (7, 3), (8, 4), (9, 4)]);
impl_dynamic!([(10, 4), (11, 4), (12, 4), (13, 4), (14, 4), (15, 4), (16, 5), (17, 5), (18, 5), (19, 5)]);
impl_dynamic!([(20, 5), (21, 5), (22, 5), (23, 5), (24, 5), (25, 5), (26, 5), (27, 5), (28, 5), (29, 5)]);
impl_dynamic!([(30, 5), (31, 5), (32, 6), (33, 6), (34, 6), (35, 6), (36, 6), (37, 6), (38, 6), (39, 6)]);
impl_dynamic!([(40, 6), (41, 6), (42, 6), (43, 6), (44, 6), (45, 6), (46, 6), (47, 6), (48, 6), (49, 6)]);
impl_dynamic!([(50, 6), (51, 6), (52, 6), (53, 6), (54, 6), (55, 6), (56, 6), (57, 6), (58, 6), (59, 6)]);
impl_dynamic!([(60, 6), (61, 6), (62, 6), (63, 6), (64, 7), (65, 7), (66, 7), (67, 7), (68, 7), (69, 7)]);
impl_dynamic!([(70, 7), (71, 7), (72, 7), (73, 7), (74, 7), (75, 7), (76, 7), (77, 7), (78, 7), (79, 7)]);
impl_dynamic!([(80, 7), (81, 7), (82, 7), (83, 7), (84, 7), (85, 7), (86, 7), (87, 7), (88, 7), (89, 7)]);
impl_dynamic!([(90, 7), (91, 7), (92, 7), (93, 7), (94, 7), (95, 7), (96, 7), (97, 7), (98, 7), (99, 7)]);

impl_dynamic!([(100, 7), (101, 7), (102, 7), (103, 7), (104, 7), (105, 7), (106, 7), (107, 7), (108, 7), (109, 7)]);
impl_dynamic!([(110, 7), (111, 7), (112, 7), (113, 7), (114, 7), (115, 7), (116, 7), (117, 7), (118, 7), (119, 7)]);
impl_dynamic!([(120, 7), (121, 7), (122, 7), (123, 7), (124, 7), (125, 7), (126, 7), (127, 7), (128, 8), (129, 8)]);
impl_dynamic!([(130, 8), (131, 8), (132, 8), (133, 8), (134, 8), (135, 8), (136, 8), (137, 8), (138, 8), (139, 8)]);
impl_dynamic!([(140, 8), (141, 8), (142, 8), (143, 8), (144, 8), (145, 8), (146, 8), (147, 8), (148, 8), (149, 8)]);
impl_dynamic!([(150, 8), (151, 8), (152, 8), (153, 8), (154, 8), (155, 8), (156, 8), (157, 8), (158, 8), (159, 8)]);
impl_dynamic!([(160, 8), (161, 8), (162, 8), (163, 8), (164, 8), (165, 8), (166, 8), (167, 8), (168, 8), (169, 8)]);
impl_dynamic!([(170, 8), (171, 8), (172, 8), (173, 8), (174, 8), (175, 8), (176, 8), (177, 8), (178, 8), (179, 8)]);
impl_dynamic!([(180, 8), (181, 8), (182, 8), (183, 8), (184, 8), (185, 8), (186, 8), (187, 8), (188, 8), (189, 8)]);
impl_dynamic!([(190, 8), (191, 8), (192, 8), (193, 8), (194, 8), (195, 8), (196, 8), (197, 8), (198, 8), (199, 8)]);

impl_dynamic!([(200, 8), (201, 8), (202, 8), (203, 8), (204, 8), (205, 8), (206, 8), (207, 8), (208, 8), (209, 8)]);
impl_dynamic!([(210, 8), (211, 8), (212, 8), (213, 8), (214, 8), (215, 8), (216, 8), (217, 8), (218, 8), (219, 8)]);
impl_dynamic!([(220, 8), (221, 8), (222, 8), (223, 8), (224, 8), (225, 8), (226, 8), (227, 8), (228, 8), (229, 8)]);
impl_dynamic!([(230, 8), (231, 8), (232, 8), (233, 8), (234, 8), (235, 8), (236, 8), (237, 8), (238, 8), (239, 8)]);
impl_dynamic!([(240, 8), (241, 8), (242, 8), (243, 8), (244, 8), (245, 8), (246, 8), (247, 8), (248, 8), (249, 8)]);
impl_dynamic!([(250, 8), (251, 8), (252, 8), (253, 8), (254, 8), (255, 8)]);


#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void1{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void2{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void3{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void4{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void5{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void6{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void7{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void8{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void9{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void10{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void11{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void12{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void13{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void14{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void15{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void16{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void17{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void18{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void19{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void20{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void21{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void22{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void23{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void24{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void25{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void26{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void27{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void28{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void29{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void30{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void31{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void32{}

#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void33{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void34{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void35{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void36{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void37{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void38{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void39{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void40{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void41{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void42{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void43{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void44{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void45{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void46{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void47{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void48{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void49{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void50{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void51{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void52{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void53{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void54{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void55{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void56{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void57{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void58{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void59{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void60{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void61{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void62{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void63{}
#[allow(non_camel_case_types)] #[derive(Copy, Clone, Default)] pub struct void64{}


macro_rules! impl_primitive_types_ux{
    {[$(($type:ident, $bits:expr)),*], $underlying_type:ident} => {$(impl_primitive_types_ux!($type, $bits, $underlying_type);)*};
    ($type:ident, $bits:expr, $underlying_type:ident) => {
        impl PrimitiveType for $type {
            const BIT_LENGTH: usize = $bits;
            fn from_bits(v: u64) -> Self {
                $type::new(v as $underlying_type)
            }
            fn to_bits(self) -> u64 {
                u64::from(self)
            }
        }
    };
}

macro_rules! impl_primitive_types_ix{
    {[$(($type:ident, $bits:expr)),*], $underlying_type:ident} => {$(impl_primitive_types_ix!($type, $bits, $underlying_type);)*};
    ($type:ident, $bits:expr, $underlying_type:ident) => {
        impl PrimitiveType for $type {
            const BIT_LENGTH: usize = $bits;
            fn from_bits(v: u64) -> Self {
                $type::new(v as $underlying_type)
            }
            fn to_bits(self) -> u64 {
                i64::from(self) as u64
            }
        }
    };
}

macro_rules! impl_primitive_types_vx{
    {[$(($type:ident, $bits:expr)),*]} => {$(impl_primitive_types_vx!($type, $bits);)*};
    ($type:ident, $bits:expr) => {
        impl PrimitiveType for $type {
            const BIT_LENGTH: usize = $bits;
            fn from_bits(_v: u64) -> Self {
                $type{}
            }
            fn to_bits(self) -> u64 {
                0
            }
        }
    };
}

impl_primitive_types_ux!([(u2, 2), (u3, 3), (u4, 4), (u5, 5), (u6, 6), (u7, 7)], u8);

impl_primitive_types_ux!([(u9, 9), (u10, 10), (u11, 11), (u12, 12), (u13, 13), (u14, 14), (u15, 15)], u16);

impl_primitive_types_ux!([(u17, 17), (u18, 18), (u19, 19), (u20, 20), (u21, 21), (u22, 22), (u23, 23), (u24, 24),
                          (u25, 25), (u26, 26), (u27, 27), (u28, 28), (u29, 29), (u30, 30), (u31, 31)], u32);

impl_primitive_types_ux!([(u33, 33), (u34, 34), (u35, 35), (u36, 36), (u37, 37), (u38, 38), (u39, 39), (u40, 40),
                          (u41, 41), (u42, 42), (u43, 43), (u44, 44), (u45, 45), (u46, 46), (u47, 47), (u48, 48),
                          (u49, 49), (u50, 50), (u51, 51), (u52, 52), (u53, 53), (u54, 54), (u55, 55), (u56, 56),
                          (u57, 57), (u58, 58), (u59, 59), (u60, 60), (u61, 61), (u62, 62), (u63, 63)], u64);



impl_primitive_types_ix!([(i2, 2), (i3, 3), (i4, 4), (i5, 5), (i6, 6), (i7, 7)], i8);

impl_primitive_types_ix!([(i9, 9), (i10, 10), (i11, 11), (i12, 12), (i13, 13), (i14, 14), (i15, 15)], i16);

impl_primitive_types_ix!([(i17, 17), (i18, 18), (i19, 19), (i20, 20), (i21, 21), (i22, 22), (i23, 23), (i24, 24),
                          (i25, 25), (i26, 26), (i27, 27), (i28, 28), (i29, 29), (i30, 30), (i31, 31)], i32);

impl_primitive_types_ix!([(i33, 33), (i34, 34), (i35, 35), (i36, 36), (i37, 37), (i38, 38), (i39, 39), (i40, 40),
                          (i41, 41), (i42, 42), (i43, 43), (i44, 44), (i45, 45), (i46, 46), (i47, 47), (i48, 48),
                          (i49, 49), (i50, 50), (i51, 51), (i52, 52), (i53, 53), (i54, 54), (i55, 55), (i56, 56),
                          (i57, 57), (i58, 58), (i59, 59), (i60, 60), (i61, 61), (i62, 62), (i63, 63)], i64);

impl_primitive_types_vx!([(void2, 2), (void3, 3), (void4, 4), (void5, 5), (void6, 6), (void7, 7), (void8, 8)]);

impl_primitive_types_vx!([(void9, 9), (void10, 10), (void11, 11), (void12, 12), (void13, 13), (void14, 14), (void15, 15), (void16, 16)]);

impl_primitive_types_vx!([(void17, 17), (void18, 18), (void19, 19), (void20, 20), (void21, 21), (void22, 22), (void23, 23), (void24, 24),
                          (void25, 25), (void26, 26), (void27, 27), (void28, 28), (void29, 29), (void30, 30), (void31, 31), (void32, 32)]);

impl_primitive_types_vx!([(void33, 33), (void34, 34), (void35, 35), (void36, 36), (void37, 37), (void38, 38), (void39, 39), (void40, 40),
                          (void41, 41), (void42, 42), (void43, 43), (void44, 44), (void45, 45), (void46, 46), (void47, 47), (void48, 48),
                          (void49, 49), (void50, 50), (void51, 51), (void52, 52), (void53, 53), (void54, 54), (void55, 55), (void56, 56),
                          (void57, 57), (void58, 58), (void59, 59), (void60, 60), (void61, 61), (void62, 62), (void63, 63), (void64, 64)]);


impl PrimitiveType for u8 {
    const BIT_LENGTH: usize =  8;
    fn from_bits(v: u64) -> Self {
        v as u8
    }
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
}

impl PrimitiveType for u16 {
    const BIT_LENGTH: usize = 16;
    fn from_bits(v: u64) -> Self {
        v as u16
    }
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
}
    
impl PrimitiveType for u32 {
    const BIT_LENGTH: usize = 32;
    fn from_bits(v: u64) -> Self {
        v as u32
    }
    fn to_bits(self) -> u64 {
        u64::from(self)
    }
}

impl PrimitiveType for u64 {
    const BIT_LENGTH: usize = 64;
    fn from_bits(v: u64) -> Self {
        v
    }
    fn to_bits(self) -> u64 {
        self
    }
}

impl PrimitiveType for i8 {
    const BIT_LENGTH: usize = 8;
    fn from_bits(v: u64) -> Self {
        v as i8
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}

impl PrimitiveType for i16 {
    const BIT_LENGTH: usize = 16;
    fn from_bits(v: u64) -> Self {
        v as i16
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}
    
impl PrimitiveType for i32 {
    const BIT_LENGTH: usize = 32;
    fn from_bits(v: u64) -> Self {
        v as i32
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}

impl PrimitiveType for i64 {
    const BIT_LENGTH: usize = 64;
    fn from_bits(v: u64) -> Self {
        v as i64
    }
    fn to_bits(self) -> u64 {
        self as u64
    }
}

impl PrimitiveType for f16 {
    const BIT_LENGTH: usize = 16;
    fn from_bits(v: u64) -> Self {
        f16::from_bits(v as u16)
    }
    fn to_bits(self) -> u64 {
        u64::from(f16::as_bits(self))
    }
}

impl PrimitiveType for f32 {
    const BIT_LENGTH: usize = 32;
    
    #[cfg_attr(feature="clippy", allow(transmute_int_to_float))]
    fn from_bits(v: u64) -> Self {
        let mut v32 = v as u32;
        const EXP_MASK: u32   = 0x7F80_0000;
        const FRACT_MASK: u32 = 0x007F_FFFF;
        if v32  & EXP_MASK == EXP_MASK && v32 & FRACT_MASK != 0 {
            v32 = unsafe { lib::core::mem::transmute(lib::core::f32::NAN) };
        }
        unsafe { lib::core::mem::transmute::<u32, f32>(v32) }
    }
    fn to_bits(self) -> u64 {
        (unsafe { lib::core::mem::transmute::<f32, u32>(self) }) as u64

            
    }
}

impl PrimitiveType for f64 {
    const BIT_LENGTH: usize = 64;
    
    #[cfg_attr(feature="clippy", allow(transmute_int_to_float))]
    fn from_bits(mut v: u64) -> Self {
        const EXP_MASK: u64   = 0x7FF0_0000_0000_0000;
        const FRACT_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
        if v & EXP_MASK == EXP_MASK && v & FRACT_MASK != 0 {
            v = unsafe { lib::core::mem::transmute(lib::core::f64::NAN) };
        }
        unsafe { lib::core::mem::transmute(v) }
    }
    fn to_bits(self) -> u64 {
        unsafe { lib::core::mem::transmute(self) }
    }
}

impl PrimitiveType for bool {
    const BIT_LENGTH: usize = 1;
    fn from_bits(v: u64) -> Self {
        v & 1 == 1
    }
    fn to_bits(self) -> u64 {
        if self {
            1
        } else {
            0
        }
    }
}
