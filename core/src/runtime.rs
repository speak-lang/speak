use super::{
    error::{Err, ErrorReason},
    eval::value::Value,
    lexer::tokenize,
    log::log_debug,
    parser::{parse, Node},
};
use crate::lexer::Tok;
use rust_i18n::t;
use std::{collections::HashMap, fmt, fs, io::BufReader};

pub const MAX_PRINT_LEN: usize = 120;

/// ValueTable is used anytime a map of names/labels to Speak Values is needed,
/// and is notably used to represent stack frames/heaps and CompositeValue dictionaries.
#[derive(Debug, Clone)]
pub struct VTable(pub HashMap<String, Value>);

impl VTable {
    fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }

    fn set(&mut self, key: String, value: Value) {
        self.0.insert(key, value);
    }
}

/// StackFrame represents the heap of variables local to a particular function call frame,
/// and recursively references other parent StackFrames internally.
#[derive(Debug, Clone)]
pub enum StackFrame {
    Frame {
        frame: VTable,
        parent_frame: Box<StackFrame>,
    },
    Nil,
}

impl StackFrame {
    /// Creates a new stack frame with the provided value table and parent stack frame.
    pub fn new(value_table: VTable, parent: StackFrame) -> Self {
        Self::Frame {
            frame: value_table,
            parent_frame: Box::new(parent),
        }
    }

    /// Pushes a frame to the stack and sets it's current frame as the parent frame.
    pub fn push_frame(&mut self, frame: VTable) {
        *self = Self::Frame {
            frame,
            parent_frame: Box::new(self.clone()),
        };
    }

    /// Pops a child frame from the stack, setting it's parent frame as the current frame.
    pub fn pop_frame(&mut self) -> Result<(), Err> {
        match self {
            StackFrame::Frame { parent_frame, .. } => {
                *self = parent_frame.as_ref().clone();
                Ok(())
            }
            StackFrame::Nil => Err(Err {
                message: t!("errors.pop_frame_e"),
                reason: ErrorReason::Assert,
            }),
        }
    }

    /// Get a value from the current stack frame; or up the parent frames.
    pub fn get(&self, name: &str) -> Option<&Value> {
        let mut frame = self;
        while let StackFrame::Frame {
            frame: item,
            parent_frame: next,
        } = frame
        {
            match item.get(name) {
                Some(val) => return Some(val),
                None => {
                    frame = next;
                }
            }
        }
        None
    }

    /// Sets a value to the provided stack frame.
    pub fn set(&mut self, name: String, val: Value) {
        if let StackFrame::Frame { frame: item, .. } = self {
            item.set(name, val)
        }
    }

    /// Updates a value in the current frame; or up the parent frames.
    pub fn up(&mut self, name: String, val: &Value) -> Result<(), Err> {
        let mut frame = self;
        while let StackFrame::Frame {
            frame: item,
            parent_frame,
        } = frame
        {
            match item.get(&name) {
                Some(_) => {
                    item.0.insert(name, val.clone());
                    return Ok(());
                }
                None => {
                    frame = parent_frame;
                }
            }
        }

        Err(Err {
            message: t!("errors.up_e", a = name),
            reason: ErrorReason::Assert,
        })
    }

    /// dumps the stack frame chain to return out.
    pub fn string(&self) -> Option<String> {
        if let StackFrame::Frame {
            frame: item,
            parent_frame: next,
        } = self
        {
            let mut entries = Vec::new();
            for (k, v) in &item.0 {
                let mut v_str = v.string();
                if v_str.len() > MAX_PRINT_LEN {
                    v_str = format!("{}...", &v_str[..MAX_PRINT_LEN])
                }
                entries.push(format!("{k} -> {v_str}"))
            }

            return Some(format!(
                "{{\n\t{}\n}} -parent-> {:?}",
                entries.join("\n\t"),
                next
            ));
        }
        None
    }
}

/// Context represents a single, isolated execution context with its global heap,
/// imports, call stack, and cwd (working directory).
#[derive(Debug)]
pub struct Context {
    /// The currently executing file's path, if any.
    pub file: Option<String>,
    /// Frame represents the Context's global heap.
    pub frame: StackFrame,

    debug_lex: bool,
    debug_parse: bool,
    debug_dump: bool,
}

impl Context {
    pub fn new(verbose: &bool) -> Self {
        Context {
            file: None,
            frame: StackFrame::new(VTable(HashMap::new()), StackFrame::Nil),
            debug_lex: *verbose,
            debug_parse: *verbose,
            debug_dump: *verbose,
        }
    }

    pub fn dump(&self) {
        if let Some(s) = self.frame.string() {
            log_debug(&format!("frame_dump:\n{}", s));
        }
    }

    /// Takes a channel of Nodes to evaluate, and executes the Speak programs defined
    /// in the syntax tree. Returning the last value of the last expression in the AST,
    /// or an error to stderr if there was a runtime error.
    pub fn eval(&mut self, nodes: Vec<Node>, dump_frame: bool) -> Result<Value, Err> {
        let len = nodes.len();
        let mut last_val = Value::Empty;

        // load runtime
        load_builtins(self)?;

        for (i, node) in nodes.into_iter().enumerate() {
            let mut node = node;
            let val = node.eval(&mut self.frame, false)?;
            if i == len - 1 {
                if dump_frame {
                    self.dump();
                }
                last_val = val;
            }
        }

        Ok(last_val)
    }

    /// Runs a Speak program defined by the buffer.
    /// This is the main way to invoke Speak programs from Rust.
    pub fn exec(
        &mut self,
        speak: &str,
        input: BufReader<&[u8]>,
    ) -> Result<(Value, Vec<Tok>, Vec<Node>), Err> {
        rust_i18n::set_locale(speak);

        let mut tokens = Vec::new();

        let mut buf = input;
        tokenize(&mut buf, &mut tokens, self.debug_lex)?;

        let mut nodes = Vec::new();

        parse(&tokens, &mut nodes, self.debug_parse)?;

        let val = self.eval(nodes.clone(), self.debug_dump)?;

        Ok((val, tokens, nodes))
    }

    /// Allows to Exec() a program file in a given context.
    pub fn exec_path(&mut self, speak: &str, path: &str) -> Result<Value, Err> {
        match fs::read(path.clone()) {
            Ok(data) => {
                self.file = Some(path.to_string());
                let (val, _, _) = self.exec(speak, BufReader::new(&data[..]))?;
                Ok(val)
            }
            Err(err) => Err(Err {
                message: t!("errors.exec_path_e", a = err),
                reason: ErrorReason::System,
            }),
        }
    }
}

/// Native function are convenience functions that come with the interpreter;
/// an example is the `println` function
#[derive(Clone)]
pub struct NativeFunction<F: Fn(&mut StackFrame, &[Value]) -> Result<Value, Err>>(
    pub String,
    pub F,
);
pub type NativeFn = NativeFunction<fn(&mut StackFrame, &[Value]) -> Result<Value, Err>>;

impl fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {{ {}: {} }}",
            t!("types.native_function"),
            t!("misc.name"),
            self.0
        )
    }
}

/// This loads up the built-in functions which come with the interpreter.
pub fn load_builtins(ctx: &mut Context) -> Result<(), Err> {
    match &mut ctx.frame {
        StackFrame::Frame { frame, .. } => {
            let key = t!("builtins.print");
            frame.set(
                key.clone(),
                Value::NativeFunction(NativeFunction(key, |_, inputs| {
                    if inputs.is_empty() {
                        return Err(Err {
                            message: t!("errors.load_builtins_e1", a = t!("builtins.print")),
                            reason: ErrorReason::Runtime,
                        });
                    }

                    print!(
                        "{}",
                        inputs[0].string().split("{}").enumerate().fold(
                            String::new(),
                            |acc, (i, x)| {
                                if i == inputs.len() - 1 {
                                    acc + x
                                } else {
                                    acc + x + &inputs[i + 1].string()
                                }
                            },
                        )
                    );

                    Ok(Value::Empty)
                })),
            );

            let key = t!("builtins.println");
            frame.set(
                key.clone(),
                Value::NativeFunction(NativeFunction(key, |_, inputs| {
                    println!(
                        "{}",
                        inputs[0].string().split("{}").enumerate().fold(
                            String::new(),
                            |acc, (i, x)| {
                                if i == inputs.len() - 1 {
                                    acc + x
                                } else {
                                    acc + x + &inputs[i + 1].string()
                                }
                            },
                        )
                    );

                    Ok(Value::Empty)
                })),
            );

            let key = t!("builtins.sprint");
            frame.set(
                key.clone(),
                Value::NativeFunction(NativeFunction(key, |_, inputs| {
                    if inputs.is_empty() {
                        return Err(Err {
                            message: t!("errors.load_builtins_e1", a = t!("builtins.sprint")),
                            reason: ErrorReason::Runtime,
                        });
                    }

                    Ok(Value::String(
                        inputs[0].string().split("{}").enumerate().fold(
                            String::new(),
                            |acc, (i, x)| {
                                if i == inputs.len() - 1 {
                                    acc + x
                                } else {
                                    acc + x + &inputs[i + 1].string()
                                }
                            },
                        ),
                    ))
                })),
            );

            let key = t!("builtins.len");
            frame.set(
                key.clone(),
                Value::NativeFunction(NativeFunction(key, |_, inputs| {
                    if inputs.len() != 1 {
                        return Err(Err {
                            message: t!("errors.load_builtins_e2", a = t!("builtins.len")),
                            reason: ErrorReason::Runtime,
                        });
                    }

                    match &inputs[0] {
                        Value::String(val) => Ok(Value::Number(val.len() as f64)),
                        Value::Array(_, val) => Ok(Value::Number(val.len() as f64)),
                        _ => Err(Err {
                            message: t!("load_builtins_e3", a = t!("builtins.len")),
                            reason: ErrorReason::Runtime,
                        }),
                    }
                })),
            );

            let key = t!("builtins.mod");
            frame.set(
                key.clone(),
                Value::NativeFunction(NativeFunction(
                    key,
                    |_stack: &mut StackFrame, inputs: &[Value]| -> Result<Value, Err> {
                        for i in inputs {
                            match i {
                                Value::String(_path) => {
                                    // TODO: load data to stack
                                    unimplemented!()
                                }
                                _ => {
                                    return Err(Err {
                                        message: t!("load_builtins_e4"),
                                        reason: ErrorReason::Runtime,
                                    });
                                }
                            }
                        }

                        Ok(Value::Empty)
                    },
                )),
            );

            Ok(())
        }

        StackFrame::Nil => Err(Err {
            message: t!("errors.load_builtins_e5"),
            reason: ErrorReason::Assert,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_frame() {
        let mut frame = StackFrame::new(VTable(HashMap::new()), StackFrame::Nil);

        // test stackframe.set(), stackframe.get()
        frame.set("a".to_string(), Value::String("hello".to_string()));
        assert!(frame
            .get("a")
            .expect("key must be present")
            .equals(Value::String("hello".to_string())));

        // test stackframe.string()
        assert_eq!(
            frame.string().expect("frame must be present"),
            "{\n\ta -> hello\n} -parent-> Nil".to_string()
        );

        // test stackframe.up(), stackframe.get()
        frame
            .up("a".to_string(), &Value::String("mutated value".to_string()))
            .expect("this calls to an already initialized variable on the stack");
        assert!(frame
            .get("a")
            .expect("key must be present")
            .equals(Value::String("mutated value".to_string())));

        // test stackframe.get, in parent frame
        let frame = StackFrame::Frame {
            frame: VTable(HashMap::new()),
            parent_frame: Box::new(frame),
        };
        assert!(frame.get("a").is_some());
        assert!(frame.get("b").is_none());
    }

    #[test]
    fn hello_world_eval() {
        let mut ctx_test = Context::new(&true);

        if let Err(err) = load_builtins(&mut ctx_test) {
            panic!("{:?}", err)
        }

        let buf_reader = BufReader::new(r#"sprint "Hello World!""#.as_bytes());
        match ctx_test.exec("en", buf_reader) {
            Ok(val) => {
                println!("{:?}\ndone!", val)
            }
            Err(err) => panic!("{:?}", err),
        }

        let cwd = std::env::current_dir().expect("there must be a wd");

        match ctx_test.exec_path("en", cwd.join("samples/hello_world.spk").to_str().expect("this path exists")) {
            Ok(val) => {
                println!("{:?}", val)
            }
            Err(err) => panic!("{:?}", err),
        }
    }
}
