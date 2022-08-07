//! This module contains buffers intended to be bound to ODBC statement handles.

mod any_column_buffer;
mod bin_column;
mod column_with_indicator;
mod columnar;
mod description;
mod indicator;
mod item;
mod text_column;

pub use self::{
    any_column_buffer::{AnyColumnBuffer, AnyColumnSliceMut, AnyColumnView, ColumnarAnyBuffer},
    bin_column::{BinColumn, BinColumnIt, BinColumnSliceMut, BinColumnView},
    column_with_indicator::{NullableSlice, NullableSliceMut},
    columnar::{ColumnBuffer, ColumnProjections, ColumnarBuffer, TextRowSet},
    description::{BufferDescription, BufferKind},
    indicator::Indicator,
    item::Item,
    text_column::{
        CharColumn, TextColumn, TextColumnIt, TextColumnSliceMut, TextColumnView, WCharColumn,
    },
};
