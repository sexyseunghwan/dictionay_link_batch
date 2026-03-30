//! DTOs for `usp_sample_proc` stored procedure.
//!
//! - [`SampleProcRequestDto`]  — input parameters passed to the procedure
//! - [`SampleProcResponseDto`] — one row returned from the procedure

/// Input parameters for `usp_sample_proc`.
#[derive(Debug)]
pub struct SampleProcRequestDto {
    /// @input_id — record identifier to look up.
    pub input_id: i32,
    /// @input_name — name filter passed to the procedure.
    pub input_name: String,
}

/// One row returned by `usp_sample_proc`.
#[derive(Debug)]
pub struct SampleProcResponseDto {
    /// result_code column — status code returned by the procedure.
    pub result_code: i32,
    /// result_message column — human-readable result message.
    pub result_message: String,
    /// result_value column — optional numeric value (NULL-able).
    pub result_value: Option<f64>,
}
