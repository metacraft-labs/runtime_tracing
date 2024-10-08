use std::cmp::Ord;
use std::ops;
use std::path::PathBuf;

use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceLowLevelEvent {
    Step(StepRecord),
    Path(PathBuf),            // should be always generated before usage, so we can stop stream at random n
    VariableName(String),     // interning new name for variables
    Variable(String),         // interning new name for variables: backward compat
    Type(TypeRecord),         // should be always generated before Value referencing it
    Value(FullValueRecord),   // full values: simpler case working even without modification support
    Function(FunctionRecord), // should be always generated before CallRecord referencing it
    Call(CallRecord),
    Return(ReturnRecord),
    Event(RecordEvent),
    CompoundValue(CompoundValueRecord),
    CellValue(CellValueRecord),
    AssignCompoundItem(AssignCompoundItemRecord),
    AssignCell(AssignCellRecord),
    VariableCell(VariableCellRecord),
    DropVariable(VariableId),
    DropLastStep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundValueRecord {
    pub value_id: ValueId,
    pub value: ValueRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellValueRecord {
    pub value_id: ValueId,
    pub value: ValueRecord,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AssignCompoundItemRecord {
    pub value_id: ValueId,
    pub index: usize,
    pub item_value_id: ValueId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignCellRecord {
    pub value_id: ValueId,
    pub new_value: ValueRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableCellRecord {
    pub variable_id: VariableId,
    pub value_id: ValueId,
}

// for now can be both just an index and
// a 64-bit pointer; think if we need
// something more general?
#[derive(Hash, Debug, Default, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct ValueId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullValueRecord {
    pub variable_id: VariableId,
    pub value: ValueRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    pub workdir: PathBuf,
    pub program: String,
    pub args: Vec<String>,
}

// call keys:

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CallKey(pub i64);

impl Into<usize> for CallKey {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl ops::Add<usize> for CallKey {
    type Output = CallKey;

    fn add(self, arg: usize) -> Self::Output {
        CallKey(self.0 + arg as i64)
    }
}

impl ops::AddAssign<usize> for CallKey {
    fn add_assign(&mut self, arg: usize) {
        self.0 += arg as i64;
    }
}

pub const NO_KEY: CallKey = CallKey(-1);

// end of call keys code

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Line(pub i64);

impl Into<usize> for Line {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Into<i64> for Line {
    fn into(self) -> i64 {
        self.0
    }
}

#[derive(Hash, Debug, Default, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(transparent)]
pub struct PathId(pub usize);

impl Into<usize> for PathId {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(transparent)]
pub struct StepId(pub i64);

impl Into<usize> for StepId {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl ops::Add<usize> for StepId {
    type Output = StepId;

    fn add(self, arg: usize) -> Self::Output {
        StepId(self.0 + arg as i64)
    }
}

impl ops::Sub<usize> for StepId {
    type Output = StepId;

    fn sub(self, arg: usize) -> Self::Output {
        StepId(self.0 - arg as i64)
    }
}

#[derive(Hash, Debug, Default, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct VariableId(pub usize);

impl Into<usize> for VariableId {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionId(pub usize);

impl Into<usize> for FunctionId {
    fn into(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRecord {
    // pub key: CallKey,
    pub function_id: FunctionId,
    pub args: Vec<FullValueRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnRecord {
    // implicit by order or explicit in some cases? pub call_key: CallKey
    pub return_value: ValueRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRecord {
    pub path_id: PathId,
    pub line: Line,
    pub name: String,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct ArgRecord {
//     pub name: String,
//     pub value: ValueRecord,
// }

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StepRecord {
    pub path_id: PathId,
    pub line: Line,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRecord {
    pub name: String,
    pub value: ValueRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeRecord {
    pub kind: TypeKind,
    pub lang_type: String,
    pub specific_info: TypeSpecificInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldTypeRecord {
    pub name: String,
    pub type_id: TypeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum TypeSpecificInfo {
    None,
    Struct { fields: Vec<FieldTypeRecord> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordEvent {
    pub kind: EventLogKind,
    pub content: String,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct TypeId(pub usize);

impl Into<usize> for TypeId {
    fn into(self) -> usize {
        self.0
    }
}

// use ValueRecord for recording custom languages
// use value::Value for interaction with existing frontend
// TODO: convert between them or
// serialize ValueRecord in a compatible way?
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum ValueRecord {
    Int {
        i: i64,
        type_id: TypeId,
    },
    Float {
        f: f64,
        type_id: TypeId,
    },
    Bool {
        b: bool,
        type_id: TypeId,
    },
    String {
        text: String,
        type_id: TypeId,
    },
    Sequence {
        elements: Vec<ValueRecord>,
        type_id: TypeId,
    },
    Struct {
        field_values: Vec<ValueRecord>,
        type_id: TypeId, // must point to Type with STRUCT kind and TypeSpecificInfo::Struct
    },
    Raw {
        r: String,
        type_id: TypeId,
    },
    Error {
        msg: String,
        type_id: TypeId,
    },
    None {
        type_id: TypeId,
    },
    Cell {
        value_id: ValueId,
    },
}

#[derive(Debug, Default, Copy, Clone, FromPrimitive, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum TypeKind {
    #[default]
    Seq,
    Set,
    HashSet,
    OrderedSet,
    Array,
    Varargs,

    Struct,

    Int,
    Float,
    String,
    CString,
    Char,
    Bool,

    Literal,

    Ref,

    Recursion,

    Raw,

    Enum,
    Enum16,
    Enum32,

    C,

    TableKind,

    Union,

    Pointer,

    Error,

    FunctionKind,

    TypeValue,

    Tuple,

    Variant,

    Html,

    None,
    NonExpanded,
    Any,
    Slice,
}

#[derive(Debug, Default, Copy, Clone, FromPrimitive, Serialize_repr, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum EventLogKind {
    #[default]
    Write,
    WriteFile,
    Read,
    ReadFile,
    // not used for now
    ReadDir,
    OpenDir,
    CloseDir,
    Socket,
    Open,
    // used for trace events
    TraceLogEvent,
}
