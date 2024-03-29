/* automatically generated by rust-bindgen 0.59.1 */

pub type size_t = ::std::os::raw::c_ulong;
pub type __int8_t = ::std::os::raw::c_schar;
pub type __uint8_t = ::std::os::raw::c_uchar;
pub type __int16_t = ::std::os::raw::c_short;
pub type __uint16_t = ::std::os::raw::c_ushort;
pub type __int32_t = ::std::os::raw::c_int;
pub type __uint32_t = ::std::os::raw::c_uint;
pub type __off_t = ::std::os::raw::c_long;
pub type __off64_t = ::std::os::raw::c_long;
pub type FILE = _IO_FILE;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _IO_marker {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _IO_codecvt {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _IO_wide_data {
    _unused: [u8; 0],
}
pub type _IO_lock_t = ::std::os::raw::c_void;
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq)]
pub struct _IO_FILE {
    pub _flags: ::std::os::raw::c_int,
    pub _IO_read_ptr: *mut ::std::os::raw::c_char,
    pub _IO_read_end: *mut ::std::os::raw::c_char,
    pub _IO_read_base: *mut ::std::os::raw::c_char,
    pub _IO_write_base: *mut ::std::os::raw::c_char,
    pub _IO_write_ptr: *mut ::std::os::raw::c_char,
    pub _IO_write_end: *mut ::std::os::raw::c_char,
    pub _IO_buf_base: *mut ::std::os::raw::c_char,
    pub _IO_buf_end: *mut ::std::os::raw::c_char,
    pub _IO_save_base: *mut ::std::os::raw::c_char,
    pub _IO_backup_base: *mut ::std::os::raw::c_char,
    pub _IO_save_end: *mut ::std::os::raw::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::std::os::raw::c_int,
    pub _flags2: ::std::os::raw::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::std::os::raw::c_ushort,
    pub _vtable_offset: ::std::os::raw::c_schar,
    pub _shortbuf: [::std::os::raw::c_char; 1usize],
    pub _lock: *mut _IO_lock_t,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::std::os::raw::c_void,
    pub __pad5: size_t,
    pub _mode: ::std::os::raw::c_int,
    pub _unused2: [::std::os::raw::c_char; 20usize],
}
#[test]
fn bindgen_test_layout__IO_FILE() {
    assert_eq!(
        ::std::mem::size_of::<_IO_FILE>(),
        216usize,
        concat!("Size of: ", stringify!(_IO_FILE))
    );
    assert_eq!(
        ::std::mem::align_of::<_IO_FILE>(),
        8usize,
        concat!("Alignment of ", stringify!(_IO_FILE))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._flags as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_flags)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_read_ptr as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_read_ptr)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_read_end as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_read_end)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_read_base as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_read_base)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_write_base as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_write_base)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_write_ptr as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_write_ptr)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_write_end as *const _ as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_write_end)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_buf_base as *const _ as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_buf_base)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_buf_end as *const _ as usize },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_buf_end)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_save_base as *const _ as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_save_base)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_backup_base as *const _ as usize },
        80usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_backup_base)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._IO_save_end as *const _ as usize },
        88usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_IO_save_end)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._markers as *const _ as usize },
        96usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_markers)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._chain as *const _ as usize },
        104usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_chain)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._fileno as *const _ as usize },
        112usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_fileno)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._flags2 as *const _ as usize },
        116usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_flags2)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._old_offset as *const _ as usize },
        120usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_old_offset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._cur_column as *const _ as usize },
        128usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_cur_column)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._vtable_offset as *const _ as usize },
        130usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_vtable_offset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._shortbuf as *const _ as usize },
        131usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_shortbuf)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._lock as *const _ as usize },
        136usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_lock)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._offset as *const _ as usize },
        144usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_offset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._codecvt as *const _ as usize },
        152usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_codecvt)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._wide_data as *const _ as usize },
        160usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_wide_data)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._freeres_list as *const _ as usize },
        168usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_freeres_list)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._freeres_buf as *const _ as usize },
        176usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_freeres_buf)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>())).__pad5 as *const _ as usize },
        184usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(__pad5)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._mode as *const _ as usize },
        192usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_mode)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_IO_FILE>()))._unused2 as *const _ as usize },
        196usize,
        concat!(
            "Offset of field: ",
            stringify!(_IO_FILE),
            "::",
            stringify!(_unused2)
        )
    );
}
impl Default for _IO_FILE {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
pub type UBool = i8;
pub type UChar = u16;
pub type UChar32 = i32;
#[repr(C)]
#[derive(Copy, Clone)]pub union UCPTrieData { pub ptr0 : * const :: std :: os :: raw :: c_void , pub ptr16 : * const u16 , pub ptr32 : * const u32 , pub ptr8 : * const u8 , }#[test]
fn bindgen_test_layout_UCPTrieData() {
    assert_eq!(
        ::std::mem::size_of::<UCPTrieData>(),
        8usize,
        concat!("Size of: ", stringify!(UCPTrieData))
    );
    assert_eq!(
        ::std::mem::align_of::<UCPTrieData>(),
        8usize,
        concat!("Alignment of ", stringify!(UCPTrieData))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrieData>())).ptr0 as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrieData),
            "::",
            stringify!(ptr0)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrieData>())).ptr16 as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrieData),
            "::",
            stringify!(ptr16)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrieData>())).ptr32 as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrieData),
            "::",
            stringify!(ptr32)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrieData>())).ptr8 as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrieData),
            "::",
            stringify!(ptr8)
        )
    );
}
impl Default for UCPTrieData {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UCPTrie {
    pub index: *const u16,
    pub data: UCPTrieData,
    pub indexLength: i32,
    pub dataLength: i32,
    pub highStart: UChar32,
    pub shifted12HighStart: u16,
    pub type_: i8,
    pub valueWidth: i8,
    pub reserved32: u32,
    pub reserved16: u16,
    pub index3NullOffset: u16,
    pub dataNullOffset: i32,
    pub nullValue: u32,
}
#[test]
fn bindgen_test_layout_UCPTrie() {
    assert_eq!(
        ::std::mem::size_of::<UCPTrie>(),
        48usize,
        concat!("Size of: ", stringify!(UCPTrie))
    );
    assert_eq!(
        ::std::mem::align_of::<UCPTrie>(),
        8usize,
        concat!("Alignment of ", stringify!(UCPTrie))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).index as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(index)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).data as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(data)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).indexLength as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(indexLength)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).dataLength as *const _ as usize },
        20usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(dataLength)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).highStart as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(highStart)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).shifted12HighStart as *const _ as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(shifted12HighStart)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).type_ as *const _ as usize },
        30usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(type_)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).valueWidth as *const _ as usize },
        31usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(valueWidth)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).reserved32 as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(reserved32)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).reserved16 as *const _ as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(reserved16)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).index3NullOffset as *const _ as usize },
        38usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(index3NullOffset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).dataNullOffset as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(dataNullOffset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UCPTrie>())).nullValue as *const _ as usize },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(UCPTrie),
            "::",
            stringify!(nullValue)
        )
    );
}
impl Default for UCPTrie {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
pub enum UCPTrieType {
    UCPTRIE_TYPE_ANY = - 1,
    UCPTRIE_TYPE_FAST = 0,
    UCPTRIE_TYPE_SMALL = 1,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
pub enum UCPTrieValueWidth {
    UCPTRIE_VALUE_BITS_ANY = - 1,
    UCPTRIE_VALUE_BITS_16 = 0,
    UCPTRIE_VALUE_BITS_32 = 1,
    UCPTRIE_VALUE_BITS_8 = 2,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct USet {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UNewTrie2 {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq)]
pub struct UTrie2 {
    pub index: *const u16,
    pub data16: *const u16,
    pub data32: *const u32,
    pub indexLength: i32,
    pub dataLength: i32,
    pub index2NullOffset: u16,
    pub dataNullOffset: u16,
    pub initialValue: u32,
    pub errorValue: u32,
    pub highStart: UChar32,
    pub highValueIndex: i32,
    pub memory: *mut ::std::os::raw::c_void,
    pub length: i32,
    pub isMemoryOwned: UBool,
    pub padding1: UBool,
    pub padding2: i16,
    pub newTrie: *mut UNewTrie2,
}
#[test]
fn bindgen_test_layout_UTrie2() {
    assert_eq!(
        ::std::mem::size_of::<UTrie2>(),
        80usize,
        concat!("Size of: ", stringify!(UTrie2))
    );
    assert_eq!(
        ::std::mem::align_of::<UTrie2>(),
        8usize,
        concat!("Alignment of ", stringify!(UTrie2))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).index as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(index)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).data16 as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(data16)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).data32 as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(data32)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).indexLength as *const _ as usize },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(indexLength)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).dataLength as *const _ as usize },
        28usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(dataLength)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).index2NullOffset as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(index2NullOffset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).dataNullOffset as *const _ as usize },
        34usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(dataNullOffset)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).initialValue as *const _ as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(initialValue)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).errorValue as *const _ as usize },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(errorValue)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).highStart as *const _ as usize },
        44usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(highStart)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).highValueIndex as *const _ as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(highValueIndex)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).memory as *const _ as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(memory)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).length as *const _ as usize },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(length)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).isMemoryOwned as *const _ as usize },
        68usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(isMemoryOwned)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).padding1 as *const _ as usize },
        69usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(padding1)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).padding2 as *const _ as usize },
        70usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(padding2)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<UTrie2>())).newTrie as *const _ as usize },
        72usize,
        concat!(
            "Offset of field: ",
            stringify!(UTrie2),
            "::",
            stringify!(newTrie)
        )
    );
}
impl Default for UTrie2 {
    fn default() -> Self {
        let mut s = ::std::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
pub enum UTargetSyntax {
    UPRV_TARGET_SYNTAX_CCODE = 0,
    UPRV_TARGET_SYNTAX_TOML = 1,
}
extern "C" {
    pub fn usrc_create(
        path: *const ::std::os::raw::c_char,
        filename: *const ::std::os::raw::c_char,
        copyrightYear: i32,
        generator: *const ::std::os::raw::c_char,
    ) -> *mut FILE;
}
extern "C" {
    pub fn usrc_createTextData(
        path: *const ::std::os::raw::c_char,
        filename: *const ::std::os::raw::c_char,
        copyrightYear: i32,
        generator: *const ::std::os::raw::c_char,
    ) -> *mut FILE;
}
extern "C" {
    pub fn usrc_writeCopyrightHeader(
        f: *mut FILE,
        prefix: *const ::std::os::raw::c_char,
        copyrightYear: i32,
    );
}
extern "C" {
    pub fn usrc_writeFileNameGeneratedBy(
        f: *mut FILE,
        prefix: *const ::std::os::raw::c_char,
        filename: *const ::std::os::raw::c_char,
        generator: *const ::std::os::raw::c_char,
    );
}
extern "C" {
    pub fn usrc_writeArray(
        f: *mut FILE,
        prefix: *const ::std::os::raw::c_char,
        p: *const ::std::os::raw::c_void,
        width: i32,
        length: i32,
        indent: *const ::std::os::raw::c_char,
        postfix: *const ::std::os::raw::c_char,
    );
}
extern "C" {
    pub fn usrc_writeUTrie2Arrays(
        f: *mut FILE,
        indexPrefix: *const ::std::os::raw::c_char,
        dataPrefix: *const ::std::os::raw::c_char,
        pTrie: *const UTrie2,
        postfix: *const ::std::os::raw::c_char,
    );
}
extern "C" {
    pub fn usrc_writeUTrie2Struct(
        f: *mut FILE,
        prefix: *const ::std::os::raw::c_char,
        pTrie: *const UTrie2,
        indexName: *const ::std::os::raw::c_char,
        dataName: *const ::std::os::raw::c_char,
        postfix: *const ::std::os::raw::c_char,
    );
}
extern "C" {
    pub fn usrc_writeUCPTrieArrays(
        f: *mut FILE,
        indexPrefix: *const ::std::os::raw::c_char,
        dataPrefix: *const ::std::os::raw::c_char,
        pTrie: *const UCPTrie,
        postfix: *const ::std::os::raw::c_char,
        syntax: UTargetSyntax,
    );
}
extern "C" {
    pub fn usrc_writeUCPTrieStruct(
        f: *mut FILE,
        prefix: *const ::std::os::raw::c_char,
        pTrie: *const UCPTrie,
        indexName: *const ::std::os::raw::c_char,
        dataName: *const ::std::os::raw::c_char,
        postfix: *const ::std::os::raw::c_char,
        syntax: UTargetSyntax,
    );
}
extern "C" {
    pub fn usrc_writeUCPTrie(
        f: *mut FILE,
        name: *const ::std::os::raw::c_char,
        pTrie: *const UCPTrie,
        syntax: UTargetSyntax,
    );
}
extern "C" {
    pub fn usrc_writeUnicodeSet(f: *mut FILE, pSet: *const USet, syntax: UTargetSyntax);
}
extern "C" {
    pub fn usrc_writeArrayOfMostlyInvChars(
        f: *mut FILE,
        prefix: *const ::std::os::raw::c_char,
        p: *const ::std::os::raw::c_char,
        length: i32,
        postfix: *const ::std::os::raw::c_char,
    );
}
extern "C" {
    pub fn usrc_writeStringAsASCII(
        f: *mut FILE,
        ptr: *const UChar,
        length: i32,
        syntax: UTargetSyntax,
    );
}
