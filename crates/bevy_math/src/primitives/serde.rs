//! This module defines serialization/deserialization for const generic arrays.
//! Unlike serde's default behavior, it supports arbitrarily large arrays.
//! The code is based on this github comment:
//! <https://github.com/serde-rs/serde/issues/1937#issuecomment-812137971>

pub(crate) mod array {
    use core::marker::PhantomData;
    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserialize, Deserializer, Serialize, Serializer,
    };

    pub fn serialize<S: Serializer, T: Serialize, const N: usize>(
        data: &[T; N],
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_tuple(N)?;
        for item in data {
            s.serialize_element(item)?;
        }
        s.end()
    }

    struct GenericArrayVisitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T, const N: usize> Visitor<'de> for GenericArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_fmt(format_args!("an array of length {N}"))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut data = [const { Option::<T>::None }; N];

            for element in data.iter_mut() {
                match (seq.next_element())? {
                    Some(val) => *element = Some(val),
                    None => return Err(serde::de::Error::invalid_length(N, &self)),
                }
            }

            let data = data.map(|value| match value {
                Some(value) => value,
                None => unreachable!(),
            });

            Ok(data)
        }
    }

    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        deserializer.deserialize_tuple(N, GenericArrayVisitor::<T, N>(PhantomData))
    }
}
