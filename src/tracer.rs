use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{
    AssignCellRecord, AssignCompoundItemRecord, CallRecord, CellValueRecord, CompoundValueRecord, EventLogKind, FullValueRecord, FunctionId,
    FunctionRecord, Line, PathId, RecordEvent, ReturnRecord, StepRecord, TraceLowLevelEvent, TraceMetadata, TypeId, TypeKind, TypeRecord,
    TypeSpecificInfo, ValueId, ValueRecord, VariableCellRecord, VariableId,
};

pub struct Tracer {
    // trace metadata:
    workdir: PathBuf,
    program: String,
    args: Vec<String>,
    // trace events
    pub events: Vec<TraceLowLevelEvent>,
    // internal tracer state:
    path_list: Vec<PathBuf>,
    function_list: Vec<(String, PathId, Line)>,

    paths: HashMap<PathBuf, PathId>,
    functions: HashMap<String, FunctionId>,
    variables: HashMap<String, VariableId>,
    types: HashMap<String, TypeId>,
}

// we ensure in start they are registered with those id-s

// pub const EXAMPLE_INT_TYPE_ID: TypeId = TypeId(0);
// pub const EXAMPLE_FLOAT_TYPE_ID: TypeId = TypeId(1);
// pub const EXAMPLE_BOOL_TYPE_ID: TypeId = TypeId(2);
// pub const EXAMPLE_STRING_TYPE_ID: TypeId = TypeId(3);
pub const NONE_TYPE_ID: TypeId = TypeId(0);
pub const NONE_VALUE: ValueRecord = ValueRecord::None { type_id: NONE_TYPE_ID };

pub const TOP_LEVEL_FUNCTION_ID: FunctionId = FunctionId(0);

impl Tracer {
    pub fn new(program: &str, args: &[String]) -> Self {
        Tracer {
            workdir: env::current_dir().expect("can access the current dir"),
            program: program.to_string(),
            args: args.to_vec(),
            events: vec![],

            path_list: vec![],
            function_list: vec![],
            paths: HashMap::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub fn start(&mut self, path: &Path, line: Line) {
        let function_id = self.ensure_function_id("<toplevel>", path, line);
        self.register_call(function_id, vec![]);
        assert!(function_id == TOP_LEVEL_FUNCTION_ID);

        // probably we let the user choose, as different languages have
        // different base types/names
        // assert!(EXAMPLE_INT_TYPE_ID == self.load_type_id(TypeKind::Int, "Int"));
        // assert!(EXAMPLE_FLOAT_TYPE_ID == self.load_type_id(TypeKind::Float, "Float"));
        // assert!(EXAMPLE_BOOL_TYPE_ID == self.load_type_id(TypeKind::Bool, "Bool"));
        // assert!(EXAMPLE_STRING_TYPE_ID == self.load_type_id(TypeKind::Bool, "String"));
        assert!(NONE_TYPE_ID == self.ensure_type_id(TypeKind::None, "None"));
    }

    pub fn ensure_path_id(&mut self, path: &Path) -> PathId {
        if !self.paths.contains_key(path) {
            self.paths.insert(path.to_path_buf(), PathId(self.paths.len()));
            self.register_path(path);
        }
        *self.paths.get(path).unwrap()
    }

    pub fn ensure_function_id(&mut self, function_name: &str, path: &Path, line: Line) -> FunctionId {
        if !self.functions.contains_key(function_name) {
            // same function names for different path line? TODO
            self.functions.insert(function_name.to_string(), FunctionId(self.functions.len()));
            self.register_function(function_name, path, line);
        }
        *self.functions.get(function_name).unwrap()
    }

    pub fn ensure_type_id(&mut self, kind: TypeKind, lang_type: &str) -> TypeId {
        if !self.types.contains_key(lang_type) {
            self.types.insert(lang_type.to_string(), TypeId(self.types.len()));
            self.register_type(kind, lang_type);
        }
        *self.types.get(lang_type).unwrap()
    }

    pub fn ensure_variable_id(&mut self, variable_name: &str) -> VariableId {
        if !self.variables.contains_key(variable_name) {
            self.variables.insert(variable_name.to_string(), VariableId(self.variables.len()));
            self.register_variable_name(variable_name);
        }
        *self.variables.get(variable_name).unwrap()
    }

    pub fn register_path(&mut self, path: &Path) {
        self.path_list.push(path.to_path_buf());
        self.events.push(TraceLowLevelEvent::Path(path.to_path_buf()));
    }

    pub fn register_function(&mut self, name: &str, path: &Path, line: Line) {
        let path_id = self.ensure_path_id(path);
        self.function_list.push((name.to_string(), path_id, line));
        self.events.push(TraceLowLevelEvent::Function(FunctionRecord {
            name: name.to_string(),
            path_id,
            line,
        }));
    }

    pub fn register_step(&mut self, path: &Path, line: Line) {
        let path_id = self.ensure_path_id(path);
        self.events.push(TraceLowLevelEvent::Step(StepRecord { path_id, line}));
    }

    pub fn register_call(&mut self, function_id: FunctionId, args: Vec<FullValueRecord>) {
        // register a step for each call, the backend expects this for
        // non-toplevel calls, so
        // we ensure it directly from register_call
        if function_id != TOP_LEVEL_FUNCTION_ID {
            for arg in &args {
                self.register_full_value(arg.variable_id, arg.value.clone());
            }
            let function = &self.function_list[function_id.0];
            self.events.push(TraceLowLevelEvent::Step(StepRecord {
                path_id: function.1,
                line: function.2,
            }));
        }
        // the actual call event:
        self.events.push(TraceLowLevelEvent::Call(CallRecord { function_id, args }));
    }

    pub fn arg(&mut self, name: &str, value: ValueRecord) -> FullValueRecord {
        let variable_id = self.ensure_variable_id(name);
        FullValueRecord { variable_id, value }
    }

    pub fn register_return(&mut self, return_value: ValueRecord) {
        self.events.push(TraceLowLevelEvent::Return(ReturnRecord { return_value }));
    }

    pub fn register_special_event(&mut self, kind: EventLogKind, content: &str) {
        self.events.push(TraceLowLevelEvent::Event(RecordEvent {
            kind,
            content: content.to_string(),
        }));
    }

    pub fn register_type(&mut self, kind: TypeKind, lang_type: &str) {
        let typ = TypeRecord {
            kind,
            lang_type: lang_type.to_string(),
            specific_info: TypeSpecificInfo::None,
        };
        self.events.push(TraceLowLevelEvent::Type(typ));
    }

    pub fn register_variable_with_full_value(&mut self, name: &str, value: ValueRecord) {
        let variable_id = self.ensure_variable_id(name);
        self.register_full_value(variable_id, value);
    }

    pub fn register_variable_name(&mut self, variable_name: &str) {
        self.events.push(TraceLowLevelEvent::VariableName(variable_name.to_string()));
    }

    pub fn register_full_value(&mut self, variable_id: VariableId, value: ValueRecord) {
        self.events.push(TraceLowLevelEvent::Value(FullValueRecord { variable_id, value }));
    }

    pub fn register_compound_value(&mut self, value_id: ValueId, value: ValueRecord) {
        self.events
            .push(TraceLowLevelEvent::CompoundValue(CompoundValueRecord { value_id, value }));
    }

    pub fn register_cell_value(&mut self, value_id: ValueId, value: ValueRecord) {
        self.events.push(TraceLowLevelEvent::CellValue(CellValueRecord { value_id, value }));
    }

    pub fn assign_compound_item(&mut self, value_id: ValueId, index: usize, item_value_id: ValueId) {
        self.events.push(TraceLowLevelEvent::AssignCompoundItem(AssignCompoundItemRecord {
            value_id,
            index,
            item_value_id,
        }));
    }
    pub fn assign_cell(&mut self, value_id: ValueId, new_value: ValueRecord) {
        self.events.push(TraceLowLevelEvent::AssignCell(AssignCellRecord { value_id, new_value }));
    }

    pub fn register_variable(&mut self, variable_name: &str, value_id: ValueId) {
        let variable_id = self.ensure_variable_id(variable_name);
        self.events
            .push(TraceLowLevelEvent::VariableCell(VariableCellRecord { variable_id, value_id }));
    }

    pub fn drop_variable(&mut self, variable_name: &str) {
        let variable_id = self.ensure_variable_id(variable_name);
        self.events.push(TraceLowLevelEvent::DropVariable(variable_id));
    }

    pub fn drop_last_step(&mut self) {
        self.events.push(TraceLowLevelEvent::DropLastStep);
    }

    pub fn store_trace_metadata(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let trace_metadata = TraceMetadata {
            program: self.program.clone(),
            args: self.args.clone(),
            workdir: self.workdir.clone(),
        };
        let json = serde_json::to_string(&trace_metadata)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn store_trace_events(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        // TODO: probably change format
        let json = serde_json::to_string(&self.events)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn store_trace_paths(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(&self.path_list)?;
        fs::write(path, json)?;
        Ok(())
    }
}
