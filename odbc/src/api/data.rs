use crate::handles::definitions::MongoHandle;
use bson::Bson;
use chrono::{
    offset::{TimeZone, Utc},
    DateTime, Datelike, Timelike,
};
use odbc_sys::{CDataType, Date, Len, Pointer, Time, Timestamp, NULL_DATA};
use std::{cmp::min, mem::size_of, str::FromStr};

pub unsafe fn format_and_return_bson(
    mongo_handle: &mut MongoHandle,
    target_type: CDataType,
    target_value_ptr: Pointer,
    buffer_length: Len,
    str_len_or_ind_ptr: *mut Len,
    data: Bson,
) {
    use std::ptr::copy_nonoverlapping;
    match data {
        // handle all NULL values:
        Bson::Array(_)
        | Bson::Document(_)
        | Bson::Null
        | Bson::RegularExpression(_)
        | Bson::JavaScriptCode(_)
        | Bson::JavaScriptCodeWithScope(_)
        | Bson::Timestamp(_)
        | Bson::Symbol(_)
        | Bson::Undefined
        | Bson::MaxKey
        | Bson::MinKey
        | Bson::DbPointer(_) => {
            *str_len_or_ind_ptr = NULL_DATA;
        }
        // handle all non-NULL values:
        // how do we support utf-16? to_utf16 I guess?
        _ => match target_type {
            CDataType::Char | CDataType::Binary => {
                let s = to_string(data);
                let l = min(buffer_length as usize, s.len());
                copy_nonoverlapping(s.as_ptr(), target_value_ptr as *mut _, l);
                *str_len_or_ind_ptr = l as Len;
            }
            CDataType::WChar => {
                let s = to_string(data);
                let data: Vec<u16> = s.encode_utf16().collect();
                let l = min(buffer_length as usize, data.len());
                copy_nonoverlapping(data.as_ptr(), target_value_ptr as *mut _, l);
                *str_len_or_ind_ptr = (l * 2) as Len;
            }
            CDataType::Bit => {
                let b = to_bool(data);
                copy_nonoverlapping(
                    &b as *const _,
                    target_value_ptr as *mut _,
                    size_of::<bool>(),
                );
                *str_len_or_ind_ptr = size_of::<bool>() as isize;
            }
            CDataType::Double => {
                let d = to_f64(data);
                copy_nonoverlapping(&d as *const _, target_value_ptr as *mut _, size_of::<f64>());
                *str_len_or_ind_ptr = size_of::<f64>() as isize;
            }
            CDataType::Float => {
                let d = to_f32(data);
                copy_nonoverlapping(&d as *const _, target_value_ptr as *mut _, size_of::<f32>());
                *str_len_or_ind_ptr = size_of::<f32>() as isize;
            }
            CDataType::SBigInt => {
                let d = to_i64(data);
                copy_nonoverlapping(
                    &d as *const _,
                    target_value_ptr as *mut _,
                    size_of::<isize>(),
                );
                *str_len_or_ind_ptr = size_of::<isize>() as isize;
            }
            CDataType::SLong => {
                let d = to_i32(data);
                copy_nonoverlapping(&d as *const _, target_value_ptr as *mut _, size_of::<i32>());
                *str_len_or_ind_ptr = size_of::<i32>() as isize;
            }
            CDataType::TimeStamp | CDataType::TypeTimestamp => {
                let dt = to_date(data);
                let out = Timestamp {
                    year: dt.year() as i16,
                    month: dt.month() as u16,
                    day: dt.day() as u16,
                    hour: dt.hour() as u16,
                    minute: dt.minute() as u16,
                    second: dt.second() as u16,
                    fraction: (dt.nanosecond() as f32 * 0.000001) as u32,
                };
                copy_nonoverlapping(
                    &out as *const _,
                    target_value_ptr as *mut _,
                    size_of::<Timestamp>(),
                );
                *str_len_or_ind_ptr = size_of::<Timestamp>() as isize;
            }
            CDataType::Time | CDataType::TypeTime => {
                let dt = to_date(data);
                let out = Time {
                    hour: dt.hour() as u16,
                    minute: dt.minute() as u16,
                    second: dt.second() as u16,
                };
                copy_nonoverlapping(
                    &out as *const _,
                    target_value_ptr as *mut _,
                    size_of::<Time>(),
                );
                *str_len_or_ind_ptr = size_of::<Time>() as isize;
            }
            CDataType::Date | CDataType::TypeDate => {
                let dt = to_date(data);
                let out = Date {
                    year: dt.year() as i16,
                    month: dt.month() as u16,
                    day: dt.day() as u16,
                };
                copy_nonoverlapping(
                    &out as *const _,
                    target_value_ptr as *mut _,
                    size_of::<Date>(),
                );
                *str_len_or_ind_ptr = size_of::<Date>() as isize;
            }
            _ => {}
        },
    }
}

fn to_string(b: Bson) -> String {
    match b {
        Bson::DateTime(d) => d.to_string(),
        Bson::String(s) => s,
        _ => b.to_string(),
    }
}

fn to_f64(b: Bson) -> f64 {
    match b {
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
        Bson::Decimal128(d) => 0.0,
        _ => 0.0,
    }
}

fn to_f32(b: Bson) -> f32 {
    match b {
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
        Bson::Decimal128(d) => 0.0,
        _ => 0.0,
    }
}

fn to_i64(b: Bson) -> i64 {
    match b {
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
        Bson::Decimal128(d) => 0,
        _ => 0,
    }
}

fn to_i32(b: Bson) -> i32 {
    match b {
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
        Bson::Decimal128(d) => 0,
        _ => 0,
    }
}

fn to_bool(b: Bson) -> bool {
    match b {
        Bson::Double(f) => f != 0.0,
        Bson::String(s) => matches!(s.as_str(), "1" | "true"),
        Bson::Boolean(b) => b,
        Bson::Int32(i) => i != 0,
        Bson::Int64(i) => i != 0,
        // TODO: Fixme when Decimal128 works.
        Bson::Decimal128(d) => false,
        _ => false,
    }
}

fn to_date(b: Bson) -> DateTime<Utc> {
    match b {
        Bson::DateTime(d) => d.into(),
        // TODO: support strings?
        _ => Utc.timestamp(0, 0),
    }
}
