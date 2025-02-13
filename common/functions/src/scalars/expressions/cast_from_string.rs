// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono_tz::Tz;
use common_arrow::arrow::bitmap::Bitmap;
use common_arrow::arrow::temporal_conversions::EPOCH_DAYS_FROM_CE;
use common_datavalues::chrono::DateTime;
use common_datavalues::chrono::Datelike;
use common_datavalues::chrono::NaiveDate;
use common_datavalues::chrono::TimeZone;
use common_datavalues::prelude::*;
use common_exception::Result;

use super::cast_with_type::arrow_cast_compute;
use super::cast_with_type::new_mutable_bitmap;
use super::cast_with_type::CastOptions;
use crate::scalars::FunctionContext;

pub fn cast_from_string(
    column: &ColumnRef,
    from_type: &DataTypeImpl,
    data_type: &DataTypeImpl,
    cast_options: &CastOptions,
    func_ctx: &FunctionContext,
) -> Result<(ColumnRef, Option<Bitmap>)> {
    let str_column = Series::remove_nullable(column);
    let str_column: &StringColumn = Series::check_get(&str_column)?;
    let size = str_column.len();
    let mut bitmap = new_mutable_bitmap(size, true);

    match data_type.data_type_id() {
        TypeID::Date => {
            let mut builder = ColumnBuilder::<i32>::with_capacity(size);

            for (row, v) in str_column.iter().enumerate() {
                if let Some(d) = string_to_date(v) {
                    builder.append((d.num_days_from_ce() - EPOCH_DAYS_FROM_CE) as i32);
                } else {
                    bitmap.set(row, false)
                }
            }

            Ok((builder.build(size), Some(bitmap.into())))
        }

        TypeID::Timestamp => {
            let mut builder = ColumnBuilder::<i64>::with_capacity(size);
            let tz = func_ctx.tz;
            for (row, v) in str_column.iter().enumerate() {
                match string_to_timestamp(v, &tz) {
                    Some(d) => {
                        builder.append(d.timestamp_micros());
                    }
                    None => bitmap.set(row, false),
                }
            }
            Ok((builder.build(size), Some(bitmap.into())))
        }
        TypeID::Boolean => {
            let mut builder = ColumnBuilder::<bool>::with_capacity(size);
            for (row, v) in str_column.iter().enumerate() {
                if v.eq_ignore_ascii_case("true".as_bytes()) {
                    builder.append(true);
                } else if v.eq_ignore_ascii_case("false".as_bytes()) {
                    builder.append(false);
                } else {
                    bitmap.set(row, false);
                }
            }
            Ok((builder.build(size), Some(bitmap.into())))
        }
        TypeID::Interval => todo!(),
        _ => arrow_cast_compute(column, from_type, data_type, cast_options, func_ctx),
    }
}

// TODO support timezone
#[inline]
pub fn string_to_timestamp(date_str: impl AsRef<[u8]>, tz: &Tz) -> Option<DateTime<Tz>> {
    let s = std::str::from_utf8(date_str.as_ref()).ok();
    s.and_then(|c| tz.datetime_from_str(c, "%Y-%m-%d %H:%M:%S%.f").ok())
}

#[inline]
pub fn string_to_date(date_str: impl AsRef<[u8]>) -> Option<NaiveDate> {
    let s = std::str::from_utf8(date_str.as_ref()).ok();
    s.and_then(|c| c.parse::<NaiveDate>().ok())
}
