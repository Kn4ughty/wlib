/// This is the code that maps to a physical button on the keyboard, irrespective of any locales
/// or anything else. To get a name for the key, use the constants defined in this file from the
/// below proc_macro `get_keys!()`. It sources its key names from "/usr/include/linux/input-event-codes.h".
///
/// This is needed because a `KeySym` press event will not always have a corresponding release
/// event. Consider: Press A. -> KeySym::a event. Then press Shift. Then releasing A ->
/// KeySym::A. a != A.
pub type RawKeyCode = u32;

/// The symbolic name of a key. Includes formatting and shift keys etc.
/// Includes functionality for turning into a unicode character etc.
///
/// Note! `key_sym.raw() != raw_key_code`
pub type KeySym = xkeysym::Keysym;

include!(concat!(env!("OUT_DIR"), "/input_codes.rs"));
