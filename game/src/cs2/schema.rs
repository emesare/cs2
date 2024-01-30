use std::ffi::{c_char, c_float, c_int, c_longlong, c_short, c_void};

use super::tier1::tshash::TSHashMap;

#[repr(C)]
pub struct SchemaClassBinding {
    parent: *mut SchemaClassBinding,
    binary_name: *const c_char,
    module_name: *const c_char,
    class_name: *const c_char,
    class_info_old_synthesized: *mut c_void,
    class_info: *mut c_void,
    this_module_binding_pointer: *mut c_void,
    schema_type: *mut SchemaType,
}

#[repr(C)]
pub union SchemaEnumValue {
    char: c_char,
    short: c_short,
    int: c_int,
    // TODO: Rename to long_long?
    value: c_longlong,
}

#[repr(C)]
pub struct SchemaEnumInfoData {
    name: *const c_char,
    value: SchemaEnumValue,
    _pad_0x10: [u8; 0x10],
}

#[repr(C)]
pub enum SchemaClassFlags {
    HasVirtualMembers = 1,
    IsAbstract = 2,
    HasTrivialConstructor = 4,
    HasTrivialDestructor = 8,
    TempHackHasNoschemaMembers = 16,
    TempHackHasConstructorLikeMethods = 32,
    TempHackHasDestructorLikeMethods = 64,
    IsNoschemaClass = 128,
}

#[repr(C)]
pub struct SchemaEnumBinding {
    binding_name: *const c_char,
    module_name: *const c_char,
    aligment: i8,
    _pad_0x19: [u8; 0x3],
    size: i16,
    flags: i16,
    enum_info: *mut SchemaEnumInfoData,
    _pad_0x28: [u8; 0x8],
    type_scope: *mut SchemaSystemTypeScope,
    _pad_0x38: [u8; 0x8],
    _unk_1: i32,
}

#[repr(C)]
pub struct SchemaStaticFieldData {
    name: *const c_char,
    ty: *mut SchemaType,
    instance: *mut c_void,
    _pad_0x18: [u8; 0x10],
}

#[repr(C)]
pub struct SchemaBaseClassInfoData {
    offset: usize,
    class: *mut SchemaClassInfoData,
}

#[repr(C)]
pub struct SchemaClassInfoData {
    _pad_0x0: [u8; 0x8],
    name: *const c_char,
    module: *mut c_char,
    size: isize,
    align: c_short,
    static_size: c_short,
    metadata_size: c_short,
    _pad_0x22: [u8; 0x6],
    fields: *mut SchemaClassFieldData,
    static_fields: *mut SchemaStaticFieldData,
    schema_parent: *mut SchemaBaseClassInfoData,
    pad_0x38: [u8; 0x10],
    metadata: *mut SchemaMetadataSetData,
}

#[repr(C)]
pub union SchemaNetworkValue {
    str: *const c_char,
    number: c_int,
    float: c_float,
    ptr: usize,
}

#[repr(C)]
pub struct SchemaMetadataEntryData {
    name: *const c_char,
    value: *mut SchemaNetworkValue,
}

#[repr(C)]
pub struct SchemaMetadataSetData {
    static_entries: SchemaMetadataEntryData,
}

#[repr(C)]
pub struct SchemaClassFieldData {
    name: *const c_char,
    ty: *mut SchemaType,
    single_inheritence_offset: i32,
    metadata_size: i32,
}

#[repr(C)]
enum SchemaTypeCategory {
    Builtin = 0,
    Pointer = 1,
    Bitfield = 2,
    FixedArray = 3,
    Atomic = 4,
    DeclaredClass = 5,
    DeclaredEnum = 6,
    None = 7,
}

#[repr(C)]
enum AtomicCategory {
    Basic,
    T,
    CollectionOfT,
    TT,
    I,
    None,
}

#[repr(C)]
union SchemaTypeUnion {
    ty: *mut SchemaType,
    class_info: *mut SchemaClassInfoData,
}

#[repr(C)]
pub struct SchemaType {
    vtable: *const c_void,
    name: *const c_char,
    type_scope: *mut SchemaSystemTypeScope,
    // TODO: Make sure these enums are 1 byte aligned.
    type_category: SchemaTypeCategory,
    atomic_category: AtomicCategory,
    info: SchemaTypeUnion,
}

#[repr(C)]
pub struct SchemaSystemTypeScope {
    _pad_0x0: [u8; 0x8],
    name: [c_char; 256],
    _pad_0x108: [u8; 0x47E],
    classes: TSHashMap<*mut SchemaClassBinding>,
    _pad_0x60e: [u8; 0x2808],
    enums: TSHashMap<*mut c_void>,
}

#[repr(C)]
pub struct SchemaSystemTypeScopeVTable {
    _unk_0: extern "C" fn(),
    _unk_1: extern "C" fn(),
    find_declared_class:
        extern "C" fn(*mut SchemaSystemTypeScope, *const c_char) -> *mut SchemaClassInfoData,
    find_declared_enum:
        extern "C" fn(*mut SchemaSystemTypeScope, *const c_char) -> *mut SchemaEnumBinding,
    _unk_4: extern "C" fn(),
    _unk_5: extern "C" fn(),
    _unk_6: extern "C" fn(),
}
