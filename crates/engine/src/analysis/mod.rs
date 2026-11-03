//! Questions the engine can answer about any circuit.

pub mod equivalence;
pub mod testbench;
pub mod timing;
pub mod truth_table;

pub use equivalence::{check_equivalence, component_circuit, Equivalence, Verdict};
pub use testbench::{bench_from_table, record_bench, run_bench, BenchReport, StepResult};
pub use timing::{critical_path, CriticalPath};
pub use truth_table::{truth_table, truth_table_with, Row, Table, MAX_TABLE_INPUTS};
