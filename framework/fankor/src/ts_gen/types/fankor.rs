use crate::prelude::{FnkInt, FnkMap, FnkRange, FnkSet, FnkString, FnkUInt, FnkURange, FnkVec};
use crate::ts_gen::types::{TsTypeGen, TsTypesCache};
use std::any::{Any, TypeId};
use std::borrow::Cow;

impl TsTypeGen for FnkInt {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("new BN(\"{}\")", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN | bigint | number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("FnkInt")
    }
}

impl TsTypeGen for FnkUInt {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("new BN(\"{}\")", self))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("BN | bigint | number")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("FnkUInt")
    }
}

impl TsTypeGen for FnkRange {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "new FnkRange(new BN(\"{}\"), new BN(\"{}\"))",
            self.from(),
            self.to()
        ))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("FnkRange")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("TFnkRange")
    }
}

impl TsTypeGen for FnkURange {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "new FnkURange(new BN(\"{}\"), new BN(\"{}\"))",
            self.from(),
            self.to()
        ))
    }

    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("FnkURange")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("TFnkURange")
    }
}

impl<'a> TsTypeGen for FnkString<'a> {
    fn value(&self) -> Cow<'static, str> {
        Cow::Owned(format!("{:?}", self))
    }
    
    fn value_type() -> Cow<'static, str> {
        Cow::Borrowed("string")
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("FnkString")
    }
}

impl<T: TsTypeGen + Any> TsTypeGen for FnkVec<T> {
    fn value(&self) -> Cow<'static, str> {
        let values = self.iter().map(|v| v.value()).collect::<Vec<_>>();

        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Owned(format!("new Uint8Array([{}])", values.join(",")))
        } else {
            Cow::Owned(format!("[{}]", values.join(",")))
        }
    }
    
    fn value_type() -> Cow<'static, str> {
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("Uint8Array")
        } else {
            Cow::Owned(format!("{}[]", T::value_type()))
        }
    }

    fn schema_name() -> Cow<'static, str> {
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("FnkByteVec")
        } else {
            Cow::Owned(format!("FnkVecSchema<{}>", T::schema_name()))
        }
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        if TypeId::of::<u8>() == TypeId::of::<T>() {
            Cow::Borrowed("FnkByteVec")
        } else {
            Cow::Owned(format!("FnkVec({})", inner_schema))
        }
    }
}

impl<T: TsTypeGen> TsTypeGen for FnkSet<T> {
    fn value(&self) -> Cow<'static, str> {
        let values = self.iter().map(|v| v.value()).collect::<Vec<_>>();
        Cow::Owned(format!("[{}]", values.join(",")))
    }
    
    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!("{}[]", T::value_type()))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!("FnkVecSchema<{}>", T::schema_name()))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_schema = T::generate_schema(registered_schemas);
        Cow::Owned(format!("FnkVec({})", inner_schema))
    }
}

impl<K: TsTypeGen, V: TsTypeGen> TsTypeGen for FnkMap<K, V> {
    fn value(&self) -> Cow<'static, str> {
        let values = self
            .iter()
            .map(|(k, v)| format!("{{ key: {}; value: {} }}", k.value(), v.value()))
            .collect::<Vec<_>>();

        Cow::Owned(format!("[{}]", values.join(",")))
    }
    
    fn value_type() -> Cow<'static, str> {
        Cow::Owned(format!("RustMap<{}, {}>", K::value_type(), V::value_type()))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!(
            "FnkMapSchema<{}, {}>",
            K::schema_name(),
            V::schema_name()
        ))
    }

    fn generate_schema(registered_schemas: &mut TsTypesCache) -> Cow<'static, str> {
        let inner_key_schema = K::generate_schema(registered_schemas);
        let inner_value_schema = V::generate_schema(registered_schemas);
        Cow::Owned(format!(
            "FnkMap({{ keySchema: {}, valueSchema: {} }})",
            inner_key_schema, inner_value_schema
        ))
    }
}
