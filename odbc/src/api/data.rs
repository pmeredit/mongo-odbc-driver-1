use crate::{errors::ODBCError, handles::definitions::MongoHandle};
use bson::Bson;
use chrono::{
    offset::{TimeZone, Utc},
    DateTime, Datelike, Timelike,
};
use odbc_sys::{CDataType, Date, Len, Pointer, Time, Timestamp};
use odbc_sys::{Char, Integer, SmallInt, SqlReturn, WChar};
use std::{cmp::min, mem::size_of, ptr::copy_nonoverlapping, str::FromStr};

const NULL: &'static str = "NULL";

/// ToCData is just used for adding methods to bson::Bson.
trait ToCData {
    fn to_string(self) -> String;
    fn to_f64(self) -> f64;
    fn to_f32(self) -> f32;
    fn to_i64(self) -> i64;
    fn to_i32(self) -> i32;
    fn to_bool(self) -> bool;
    fn to_date(self) -> DateTime<Utc>;
}

impl ToCData for Bson {
    fn to_string(self) -> String {
        match self {
            Bson::Null => NULL.to_string(),
            Bson::Undefined => NULL.to_string(),
            Bson::String(s) => s,
            Bson::Decimal128(d) => ODBCDecimal128::new(d.bytes()).to_string(),
            Bson::Array()
            | Bson::Binary(_)
            | Bson::DateTime(_)
            | Bson::DbPointer(_)
            | Bson::Document(_)
            | Bson::JavaScriptCode(_)
            | Bson::JavaScriptCodeWithScope(_)
            | Bson::MaxKey
            | Bson::MinKey
            | Bson::ObjectId(_)
            | Bson::RegularExpression(_)
            | Bson::Symbol(_)
            | Bson::Timestamp(_) => self.into_canonical_extjson().to_string(),
        }
    }

    fn to_f64(self) -> f64 {
        match self {
            Bson::DateTime(d) => d.timestamp_millis() as f64,
            Bson::Double(f) => f,
            Bson::String(s) => f64::from_str(&s).unwrap_or(0.0),
            Bson::Boolean(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            Bson::Int32(i) => i as f64,
            Bson::Int64(i) => i as f64,
            // TODO: Fixme when Decimal128 works.
            Bson::Decimal128(_d) => 0.0,
            _ => 0.0,
        }
    }

    fn to_f32(self) -> f32 {
        match self {
            Bson::DateTime(d) => d.timestamp_millis() as f32,
            Bson::Double(f) => f as f32,
            Bson::String(s) => f32::from_str(&s).unwrap_or(0.0),
            Bson::Boolean(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            Bson::Int32(i) => i as f32,
            Bson::Int64(i) => i as f32,
            // TODO: Fixme when Decimal128 works.
            Bson::Decimal128(_d) => 0.0,
            _ => 0.0,
        }
    }

    fn to_i64(self) -> i64 {
        match self {
            Bson::DateTime(d) => d.timestamp_millis(),
            Bson::Double(f) => f as i64,
            Bson::String(s) => i64::from_str(&s).unwrap_or(0),
            Bson::Boolean(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
            Bson::Int32(i) => i as i64,
            Bson::Int64(i) => i,
            // TODO: Fixme when Decimal128 works.
            Bson::Decimal128(_d) => 0,
            _ => 0,
        }
    }

    fn to_i32(self) -> i32 {
        match self {
            Bson::DateTime(d) => d.timestamp_millis() as i32,
            Bson::Double(f) => f as i32,
            Bson::String(s) => i32::from_str(&s).unwrap_or(0),
            Bson::Boolean(b) => {
                if b {
                    1
                } else {
                    0
                }
            }
            Bson::Int32(i) => i,
            Bson::Int64(i) => i as i32,
            // TODO: Fixme when Decimal128 works.
            Bson::Decimal128(_d) => 0,
            _ => 0,
        }
    }

    fn to_bool(self) -> bool {
        match self {
            Bson::Double(f) => f != 0.0,
            Bson::String(s) => matches!(s.as_str(), "1" | "true"),
            Bson::Boolean(b) => b,
            Bson::Int32(i) => i != 0,
            Bson::Int64(i) => i != 0,
            // TODO: Fixme when Decimal128 works.
            Bson::Decimal128(_d) => false,
            _ => false,
        }
    }

    fn to_date(self) -> DateTime<Utc> {
        match self {
            Bson::DateTime(d) => d.into(),
            // TODO: support strings?
            _ => Utc.timestamp(0, 0),
        }
    }
}

pub unsafe fn format_and_return_bson(
    mongo_handle: &mut MongoHandle,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_len: Len,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) -> SqlReturn {
    match target_type {
        CDataType::Char | CDataType::Binary => set_output_string(
            &data.to_string(),
            target_value_ptr as *mut _,
            buffer_len as usize,
            str_len_or_ind_ptr as *mut _,
        ),
        CDataType::WChar => set_output_wstring(
            &data.to_string(),
            target_value_ptr as *mut _,
            buffer_len as usize,
            str_len_or_ind_ptr as *mut _,
        ),
        CDataType::Bit => set_output_fixed_data(
            &data.to_bool(),
            target_value_ptr,
            buffer_len,
            str_len_or_ind_ptr,
        ),
        CDataType::Double => set_output_fixed_data(
            &data.to_f64(),
            target_value_ptr,
            buffer_len,
            str_len_or_ind_ptr,
        ),
        CDataType::Float => set_output_fixed_data(
            &data.to_f32(),
            target_value_ptr,
            buffer_len,
            str_len_or_ind_ptr,
        ),
        CDataType::SBigInt | CDataType::Numeric => set_output_fixed_data(
            &data.to_i64(),
            target_value_ptr,
            buffer_len,
            str_len_or_ind_ptr,
        ),
        CDataType::SLong => set_output_fixed_data(
            &data.to_i32(),
            target_value_ptr,
            buffer_len,
            str_len_or_ind_ptr,
        ),
        CDataType::TimeStamp | CDataType::TypeTimestamp => {
            let dt = data.to_date();
            let data = Timestamp {
                year: dt.year() as i16,
                month: dt.month() as u16,
                day: dt.day() as u16,
                hour: dt.hour() as u16,
                minute: dt.minute() as u16,
                second: dt.second() as u16,
                fraction: (dt.nanosecond() as f32 * 0.000001) as u32,
            };
            set_output_fixed_data(&data, target_value_ptr, buffer_len, str_len_or_ind_ptr)
        }
        CDataType::Time | CDataType::TypeTime => {
            let dt = data.to_date();
            let data = Time {
                hour: dt.hour() as u16,
                minute: dt.minute() as u16,
                second: dt.second() as u16,
            };
            set_output_fixed_data(&data, target_value_ptr, buffer_len, str_len_or_ind_ptr)
        }
        CDataType::Date | CDataType::TypeDate => {
            let dt = data.to_date();
            let data = Date {
                year: dt.year() as i16,
                month: dt.month() as u16,
                day: dt.day() as u16,
            };
            set_output_fixed_data(&data, target_value_ptr, buffer_len, str_len_or_ind_ptr)
        }
        _ => {
            mongo_handle.add_diag_info(ODBCError::Unimplemented("unimplemented data type"));
            SqlReturn::ERROR
        }
    }
}

///
/// input_wtext_to_string converts an input cstring to a rust String.
/// It assumes nul termination if the supplied length is negative.
///
/// # Safety
/// This converts raw C-pointers to rust Strings, which requires unsafe operations
///
#[allow(clippy::uninit_vec)]
pub unsafe fn input_wtext_to_string(text: *const WChar, len: usize) -> String {
    if (len as isize) < 0 {
        let mut dst = Vec::new();
        let mut itr = text;
        {
            while *itr != 0 {
                dst.push(*itr);
                itr = itr.offset(1);
            }
        }
        return String::from_utf16_lossy(&dst);
    }

    let mut dst = Vec::with_capacity(len);
    dst.set_len(len);
    copy_nonoverlapping(text, dst.as_mut_ptr(), len);
    String::from_utf16_lossy(&dst)
}

///
/// set_sql_state writes the given sql state to the [`output_ptr`].
///
/// # Safety
/// This writes to a raw C-pointer
///
pub unsafe fn set_sql_state(sql_state: &str, output_ptr: *mut WChar) {
    if output_ptr.is_null() {
        return;
    }
    let sql_state = &format!("{}\0", sql_state);
    let state_u16 = sql_state.encode_utf16().collect::<Vec<u16>>();
    copy_nonoverlapping(state_u16.as_ptr(), output_ptr, 6);
}

///
/// set_output_wstring writes [`message`] to the *WChar [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the message should be truncated
/// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
/// should be stored in [`text_length_ptr`].
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn set_output_wstring(
    message: &str,
    output_ptr: *mut WChar,
    buffer_len: usize,
    text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    if output_ptr.is_null() {
        if !text_length_ptr.is_null() {
            *text_length_ptr = 0 as SmallInt;
        } else {
            // If the output_ptr is NULL, we should still return the length of the message.
            *text_length_ptr = message.encode_utf16().count() as i16;
        }
        return SqlReturn::SUCCESS_WITH_INFO;
    }
    // Check if the entire message plus a null terminator can fit in the buffer;
    // we should truncate the message if it's too long.
    let mut message_u16 = message.encode_utf16().collect::<Vec<u16>>();
    let message_len = message_u16.len();
    let num_chars = min(message_len + 1, buffer_len);
    // It is possible that no buffer space has been allocated.
    if num_chars == 0 {
        return SqlReturn::SUCCESS_WITH_INFO;
    }
    message_u16.resize(num_chars - 1, 0);
    message_u16.push('\u{0}' as u16);
    copy_nonoverlapping(message_u16.as_ptr(), output_ptr, num_chars);
    // Store the number of characters in the message string, excluding the
    // null terminator, in text_length_ptr
    if !text_length_ptr.is_null() {
        *text_length_ptr = (num_chars - 1) as SmallInt;
    }
    if num_chars < message_len {
        SqlReturn::SUCCESS_WITH_INFO
    } else {
        SqlReturn::SUCCESS
    }
}

///
/// set_output_fixed_data writes [`data`], which must be a fixed sized type, to the Pointer [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in bytes; the message should be truncated
/// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
/// should be stored in [`text_length_ptr`].
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn set_output_fixed_data<T>(
    data: &T,
    output_ptr: Pointer,
    buffer_len: Len,
    data_len_ptr: *mut Len,
) -> SqlReturn {
    if output_ptr.is_null() {
        if !data_len_ptr.is_null() {
            *data_len_ptr = 0 as Len;
        } else {
            // If the output_ptr is NULL, we should still return the length of the message.
            *data_len_ptr = size_of::<T>() as isize;
        }
        return SqlReturn::SUCCESS_WITH_INFO;
    }
    copy_nonoverlapping(data as *const _, output_ptr as *mut _, 1);
    SqlReturn::SUCCESS
}

///
/// set_output_wstring writes [`message`] to the *Char [`output_ptr`]. [`buffer_len`] is the
/// length of the [`output_ptr`] buffer in characters; the message should be truncated
/// if it is longer than the buffer length. The number of characters written to [`output_ptr`]
/// should be stored in [`text_length_ptr`].
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn set_output_string(
    message: &str,
    output_ptr: *mut Char,
    buffer_len: usize,
    text_length_ptr: *mut SmallInt,
) -> SqlReturn {
    if output_ptr.is_null() {
        if !text_length_ptr.is_null() {
            *text_length_ptr = 0 as SmallInt;
        } else {
            // If the output_ptr is NULL, we should still return the length of the message.
            *text_length_ptr = message.len() as i16;
        }
        return SqlReturn::SUCCESS_WITH_INFO;
    }
    // Check if the entire message plus a null terminator can fit in the buffer;
    // we should truncate the message if it's too long.
    // Note, we also assume this is valid ascii
    let mut message_u8 = message.bytes().collect::<Vec<u8>>();
    let message_len = message_u8.len();
    let num_chars = min(message_len + 1, buffer_len);
    // It is possible that no buffer space has been allocated.
    if num_chars == 0 {
        return SqlReturn::SUCCESS_WITH_INFO;
    }
    message_u8.resize(num_chars - 1, 0);
    message_u8.push('\u{0}' as u8);
    copy_nonoverlapping(message_u8.as_ptr(), output_ptr, num_chars);
    // Store the number of characters in the message string, excluding the
    // null terminator, in text_length_ptr
    if !text_length_ptr.is_null() {
        *text_length_ptr = (num_chars - 1) as SmallInt;
    }
    if num_chars < message_len {
        SqlReturn::SUCCESS_WITH_INFO
    } else {
        SqlReturn::SUCCESS
    }
}

///
/// get_diag_rec copies the given ODBC error's diagnostic information
/// into the provided pointers.
///
/// # Safety
/// This writes to multiple raw C-pointers
///
pub unsafe fn get_diag_rec(
    error: &ODBCError,
    state: *mut WChar,
    message_text: *mut WChar,
    buffer_length: SmallInt,
    text_length_ptr: *mut SmallInt,
    native_error_ptr: *mut Integer,
) -> SqlReturn {
    if !native_error_ptr.is_null() {
        *native_error_ptr = error.get_native_err_code();
    }
    set_sql_state(error.get_sql_state(), state);
    let message = format!("{}", error);
    set_output_string(
        &message,
        message_text,
        buffer_length as usize,
        text_length_ptr,
    )
}

///
/// unsupported_function is a helper function for correctly setting the state for
/// unsupported functions.
///
pub fn unsupported_function(handle: &mut MongoHandle, name: &'static str) -> SqlReturn {
    handle.clear_diagnostics();
    handle.add_diag_info(ODBCError::Unimplemented(name));
    SqlReturn::ERROR
}

///
/// set_str_length writes the given length to [`string_length_ptr`].
///
/// # Safety
/// This writes to a raw C-pointers
///
pub unsafe fn set_str_length(string_length_ptr: *mut Integer, length: Integer) {
    if !string_length_ptr.is_null() {
        *string_length_ptr = length
    }
}
