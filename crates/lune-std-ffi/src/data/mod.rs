use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

mod box_data;
mod callable_data;
mod closure_data;
mod helper;
mod lib_data;
mod ref_data;

pub use self::{
    box_data::BoxData,
    callable_data::CallableData,
    closure_data::ClosureData,
    lib_data::LibData,
    ref_data::{create_nullref, RefBounds, RefData, RefFlag, UNSIZED_BOUNDS},
};
use crate::ffi::FfiData;

mod association_names {
    pub const REF_INNER: &str = "__ref_inner";
    pub const SYM_INNER: &str = "__syn_inner";
}

pub enum FfiDataRef {
    Box(LuaUserDataRef<BoxData>),
    Ref(LuaUserDataRef<RefData>),
    Closure(LuaUserDataRef<ClosureData>),
}

impl FfiData for FfiDataRef {
    fn check_inner_boundary(&self, offset: isize, size: usize) -> bool {
        match self {
            FfiDataRef::Box(d) => d.check_inner_boundary(offset, size),
            FfiDataRef::Ref(d) => d.check_inner_boundary(offset, size),
            FfiDataRef::Closure(d) => d.check_inner_boundary(offset, size),
        }
    }
    unsafe fn get_inner_pointer(&self) -> *mut () {
        match self {
            FfiDataRef::Box(d) => d.get_inner_pointer(),
            FfiDataRef::Ref(d) => d.get_inner_pointer(),
            FfiDataRef::Closure(d) => d.get_inner_pointer(),
        }
    }
    fn is_writable(&self) -> bool {
        match self {
            FfiDataRef::Box(d) => d.is_writable(),
            FfiDataRef::Ref(d) => d.is_writable(),
            FfiDataRef::Closure(d) => d.is_writable(),
        }
    }
    fn is_readable(&self) -> bool {
        match self {
            FfiDataRef::Box(d) => d.is_readable(),
            FfiDataRef::Ref(d) => d.is_readable(),
            FfiDataRef::Closure(d) => d.is_readable(),
        }
    }
}

pub trait GetFfiData {
    fn get_ffi_data(&self) -> LuaResult<FfiDataRef>;
    fn is_ffi_data(&self) -> bool;
}
impl GetFfiData for LuaAnyUserData {
    fn get_ffi_data(&self) -> LuaResult<FfiDataRef> {
        if self.is::<BoxData>() {
            Ok(FfiDataRef::Box(self.borrow::<BoxData>()?))
        } else if self.is::<RefData>() {
            Ok(FfiDataRef::Ref(self.borrow::<RefData>()?))
        } else if self.is::<ClosureData>() {
            Ok(FfiDataRef::Closure(self.borrow::<ClosureData>()?))
        } else {
            let config = ValueFormatConfig::new();
            Err(LuaError::external(format!(
                "Expected FfiBox, FfiRef or ClosureData. got {}",
                pretty_format_value(&LuaValue::UserData(self.to_owned()), &config)
            )))
        }
    }
    fn is_ffi_data(&self) -> bool {
        self.is::<BoxData>() | self.is::<RefData>() | self.is::<ClosureData>()
    }
}
impl GetFfiData for LuaValue {
    fn get_ffi_data(&self) -> LuaResult<FfiDataRef> {
        self.as_userdata()
            .ok_or_else(|| {
                let config = ValueFormatConfig::new();
                LuaError::external(format!(
                    "Expected FfiBox, FfiRef or ClosureData. got {}",
                    pretty_format_value(self, &config)
                ))
            })?
            .get_ffi_data()
    }
    fn is_ffi_data(&self) -> bool {
        self.as_userdata().map_or(false, GetFfiData::is_ffi_data)
    }
}
