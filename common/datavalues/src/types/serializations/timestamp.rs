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

use chrono::DateTime;
use chrono_tz::Tz;
use common_exception::*;
use opensrv_clickhouse::types::column::ArcColumnWrapper;
use opensrv_clickhouse::types::column::ColumnFrom;
use serde_json::Value;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct TimestampSerializer {
    tz: Tz,
}

impl Default for TimestampSerializer {
    fn default() -> Self {
        let tz = "UTC".parse::<Tz>().unwrap();
        Self { tz }
    }
}

impl TimestampSerializer {
    pub fn to_timestamp(&self, value: &i64) -> DateTime<Tz> {
        value.to_timestamp(&self.tz)
    }
}

const TIME_FMT: &str = "%Y-%m-%d %H:%M:%S";

impl TypeSerializer for TimestampSerializer {
    fn serialize_value(&self, value: &DataValue) -> Result<String> {
        let value = DFTryFrom::try_from(value.clone())?;
        let dt = self.to_timestamp(&value);
        Ok(dt.format(TIME_FMT).to_string())
    }

    fn serialize_column(&self, column: &ColumnRef) -> Result<Vec<String>> {
        let column: &PrimitiveColumn<i64> = Series::check_get(column)?;
        let result: Vec<String> = column
            .iter()
            .map(|v| {
                let dt = self.to_timestamp(v);
                dt.format(TIME_FMT).to_string()
            })
            .collect();
        Ok(result)
    }

    fn serialize_json(&self, column: &ColumnRef) -> Result<Vec<Value>> {
        let array: &PrimitiveColumn<i64> = Series::check_get(column)?;
        let result: Vec<Value> = array
            .iter()
            .map(|v| {
                let dt = self.to_timestamp(v);
                serde_json::to_value(dt.format(TIME_FMT).to_string()).unwrap()
            })
            .collect();
        Ok(result)
    }

    fn serialize_clickhouse_format(
        &self,
        column: &ColumnRef,
    ) -> Result<opensrv_clickhouse::types::column::ArcColumnData> {
        let array: &PrimitiveColumn<i64> = Series::check_get(column)?;
        let values: Vec<DateTime<Tz>> = array.iter().map(|v| self.to_timestamp(v)).collect();
        Ok(Vec::column_from::<ArcColumnWrapper>(values))
    }
}
