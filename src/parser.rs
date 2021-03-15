use rand::prelude::*;
use std::collections::HashMap;

pub fn initial_data() -> Vec<String> {
    let mut commands: Vec<String> = Vec::new();

    commands.push(String::from("initialize sp 256"));
    commands.push(String::from("call Sys.init 0"));

    parse_content(commands, String::from("initialize ASM file"))
}

pub fn parse_content(content: Vec<String>, filename: String) -> Vec<String> {
    let actions = build_actions();
    let mut result: Vec<String> = Vec::new();
    let mut instruction = VMInstruction::new(&filename);

    result.push(format!("// New file: {}", &filename));

    for line in content {
        result.push(format!("// {}", line));

        &instruction.update(&line);

        let action = actions
            .get(&instruction.command)
            .expect("Invalid action required!");

        result.extend(action(&instruction));
    }

    result
}

struct VMInstruction {
    pub command: String,
    pub detail: String,
    pub value: String,
    pub function_name: String,
    pub filename: String,
}

impl VMInstruction {
    pub fn new(filename: &str) -> VMInstruction {
        VMInstruction {
            command: String::new(),
            detail: String::new(),
            value: String::new(),
            function_name: String::new(),
            filename: String::from(filename),
        }
    }

    pub fn update(&mut self, line: &String) {
        let (command, detail, value) = VMInstruction::split_command(&line);

        let function_name = if command == "function" {
            detail.clone()
        } else {
            self.function_name.clone()
        };

        self.command = command;
        self.detail = detail;
        self.value = value;
        self.function_name = function_name;
    }

    pub fn build_label(&self) -> String {
        format!("{}${}", self.function_name, self.detail)
    }

    pub fn build_static_label(&self) -> String {
        format!("{}${}", self.filename, self.value)
    }

    pub fn build_label_from(&self, value: &str) -> String {
        format!("{}${}", self.function_name, value)
    }

    pub fn build_random_label_from(&self, value: &str) -> String {
        let random_label = format!("{}", rand::thread_rng().gen::<u32>());
        format!("{}${}.{}", self.function_name, value, random_label)
    }

    pub fn build_function_name(&self) -> String {
        format!("{}", self.detail)
    }

    fn split_command(line: &String) -> (String, String, String) {
        let command: Vec<&str> = line.split(' ').collect();

        if command.len() == 1 {
            return (
                String::from(command[0].trim()),
                String::new(),
                String::new(),
            );
        }

        if command.len() == 2 {
            return (
                String::from(command[0].trim()),
                String::from(command[1].trim()),
                String::new(),
            );
        }

        (
            String::from(command[0].trim()),
            String::from(command[1].trim()),
            String::from(command[2].trim()),
        )
    }
}

type Callback = fn(&VMInstruction) -> Vec<String>;

fn build_actions() -> HashMap<String, Callback> {
    let mut actions: HashMap<String, Callback> = HashMap::new();

    actions.insert(String::from("push"), push_action);
    actions.insert(String::from("pop"), pop_action);

    actions.insert(String::from("add"), add_action);
    actions.insert(String::from("sub"), sub_action);
    actions.insert(String::from("neg"), neg_action);

    actions.insert(String::from("eq"), eq_action);
    actions.insert(String::from("lt"), lt_action);
    actions.insert(String::from("gt"), gt_action);

    actions.insert(String::from("and"), and_action);
    actions.insert(String::from("or"), or_action);
    actions.insert(String::from("not"), not_action);

    actions.insert(String::from("label"), label_action);
    actions.insert(String::from("goto"), goto_action);
    actions.insert(String::from("if-goto"), ifgoto_action);

    actions.insert(String::from("function"), function_action);
    actions.insert(String::from("return"), return_action);
    actions.insert(String::from("call"), call_action);

    actions.insert(String::from("initialize"), initialize_action);

    actions
}

fn push_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    // use value information
    match instruction.detail.as_str() {
        "local" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_value_from_segment_plus_d("LCL");
            builder.push_to_stack();
        }
        "argument" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_value_from_segment_plus_d("ARG");
            builder.push_to_stack();
        }
        "this" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_value_from_segment_plus_d("THIS");
            builder.push_to_stack();
        }
        "that" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_value_from_segment_plus_d("THAT");
            builder.push_to_stack();
        }
        "constant" => {
            builder.move_value_to_d(&instruction.value);
            builder.push_to_stack();
        }
        "temp" => {
            builder.move_value_to_d(&instruction.value);

            builder.at("5");
            builder.d_plus_a_address_to_d();

            builder.push_to_stack();
        }
        "static" => {
            builder.at(instruction.build_static_label().as_str());
            builder.m_to_d();
            builder.push_to_stack();
        }
        "pointer" => {
            let parsed_value = if instruction.value == "0" {
                "THIS"
            } else {
                "THAT"
            };

            builder.at(parsed_value);
            builder.m_to_d();
            builder.push_to_stack();
        }
        _ => panic!(format!("Invalid memory location! {}", instruction.detail)),
    }

    builder.parsed_content()
}

fn pop_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();
    let temp_label = instruction.build_label_from("tmp");

    // use value information
    match instruction.detail.as_str() {
        "local" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_address_from_segment_plus_d("LCL");
            builder.d_to_tmp(&temp_label);
            builder.pop_from_stack_to(&temp_label);
        }
        "argument" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_address_from_segment_plus_d("ARG");
            builder.d_to_tmp(&temp_label);
            builder.pop_from_stack_to(&temp_label);
        }
        "this" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_address_from_segment_plus_d("THIS");
            builder.d_to_tmp(&temp_label);
            builder.pop_from_stack_to(&temp_label);
        }
        "that" => {
            builder.move_value_to_d(&instruction.value);
            builder.get_address_from_segment_plus_d("THAT");
            builder.d_to_tmp(&temp_label);
            builder.pop_from_stack_to(&temp_label);
        }
        "temp" => {
            builder.move_value_to_d(&instruction.value);
            builder.at("5");
            builder.d_plus_a_to_d();
            builder.d_to_tmp(&temp_label);
            builder.pop_from_stack_to(&temp_label);
        }
        "static" => {
            builder.pop_from_stack_to_d();
            builder.at(instruction.build_static_label().as_str());
            builder.d_to_m();
        }
        "pointer" => {
            let parsed_value = if instruction.value == "0" {
                "THIS"
            } else {
                "THAT"
            };

            builder.pop_from_stack_to_d();
            builder.at(parsed_value);
            builder.d_to_m();
        }
        _ => panic!(format!("Invalid memory location! {}", instruction.detail)),
    }

    builder.parsed_content()
}

fn add_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.d_plus_m_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn sub_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn eq_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_d();
    builder.compare_with_d("JEQ", &instruction.build_random_label_from("IS_TRUE"));
    builder.advance_sp();

    builder.parsed_content()
}

fn lt_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_d();
    builder.compare_with_d("JLT", &instruction.build_random_label_from("IS_TRUE"));
    builder.advance_sp();

    builder.parsed_content()
}

fn gt_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_d();
    builder.compare_with_d("JGT", &instruction.build_random_label_from("IS_TRUE"));
    builder.advance_sp();

    builder.parsed_content()
}

fn and_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_and_d_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn or_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_or_d_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn not_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack();
    builder.not_m_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn neg_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack();
    builder.neg_m_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn label_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.label(&instruction.build_label());

    builder.parsed_content()
}

fn goto_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.goto_label(&instruction.build_label());

    builder.parsed_content()
}

fn ifgoto_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.jump_to_label_if_d_neq(&instruction.build_label());

    builder.parsed_content()
}

fn function_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();
    let while_label = instruction.build_random_label_from("WHILE");
    let break_label = instruction.build_random_label_from("BREAK");

    builder.label(&instruction.build_function_name()); // set the label for the function

    builder.move_value_to_d(&instruction.value);
    builder.push_to_stack();

    builder.label(&while_label); // while d != 0

    builder.pop_from_stack_to_d();
    builder.jump_to_label_if_d_eq(&break_label);
    builder.push_to_stack_zero();
    builder.d_less_one_to_d(); // d--
    builder.push_to_stack();

    builder.goto_label(&while_label);

    builder.label(&break_label); // end while

    builder.parsed_content()
}

fn return_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    let temp_return_value = "R13";
    let temp_return_addr = "R14";
    let temp_sp = "R15";

    builder.pop_from_stack_to_d();
    builder.at(temp_return_value);
    builder.d_to_m(); // save return value to temp.return.value

    builder.get_value_at("ARG");
    builder.at(temp_sp);
    builder.d_to_m(); // save SP original value to temp.sp

    builder.get_value_at("LCL");
    builder.at("SP");
    builder.d_to_m(); // return pointer to end of backup values

    builder.pop_from_stack_to_d();
    builder.at("THAT");
    builder.d_to_m(); // restore THAT value

    builder.pop_from_stack_to_d();
    builder.at("THIS");
    builder.d_to_m(); // restore THIS value

    builder.pop_from_stack_to_d();
    builder.at("ARG");
    builder.d_to_m(); // restore ARG value

    builder.pop_from_stack_to_d();
    builder.at("LCL");
    builder.d_to_m(); // restore LCL value

    builder.pop_from_stack_to_d();
    builder.at(temp_return_addr);
    builder.d_to_m(); // restore ARG value

    builder.get_value_at(temp_sp);
    builder.at("SP");
    builder.d_to_m(); // restore SP value

    builder.get_value_at(temp_return_value);
    builder.push_to_stack(); // push return value to stack

    builder.goto_value_at(temp_return_addr); // go back to flow

    builder.parsed_content()
}

fn call_action(instruction: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();
    let random_jump: String = instruction.build_random_label_from("ret");

    builder.label(&random_jump); // get a return point at d
    builder.move_value_to_d(&random_jump);
    builder.at("51");
    builder.d_plus_a_to_d();
    builder.push_to_stack(); // push return addr to stack

    builder.get_value_at("LCL");
    builder.push_to_stack(); // push LCL addr to stack

    builder.get_value_at("ARG");
    builder.push_to_stack(); // push ARG addr to stack

    builder.get_value_at("THIS");
    builder.push_to_stack(); // push THIS addr to stack

    builder.get_value_at("THAT");
    builder.push_to_stack(); // push THAT addr to stack

    builder.move_value_to_d("5");
    builder.at(&instruction.value);
    builder.d_plus_a_to_d();
    builder.at("SP");
    builder.m_less_d_to_d();
    builder.at("ARG");
    builder.d_to_m(); // Move ARGS to first argument

    builder.get_value_at("SP");
    builder.at("LCL");
    builder.d_to_m(); // Move LCL to first empty SP (will be filled by function)

    builder.goto_label(&instruction.build_function_name()); // go to function

    builder.parsed_content()
}

fn initialize_action(_: &VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.move_value_to_d("256");
    builder.at("SP");
    builder.d_to_m();

    builder.parsed_content()
}

struct AssemblerCommandBuilder {
    result: Vec<String>,
}

impl AssemblerCommandBuilder {
    pub fn new() -> AssemblerCommandBuilder {
        AssemblerCommandBuilder { result: Vec::new() }
    }

    pub fn parsed_content(&self) -> Vec<String> {
        self.result.clone()
    }

    pub fn label(&mut self, value: &str) {
        self.result.push(format!("({})", value));
    }

    pub fn goto_label(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("0;JMP"));
    }

    pub fn jump_to_label_if_d_neq(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("D;JNE"));
    }

    pub fn jump_to_label_if_d_eq(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("D;JEQ"));
    }

    pub fn at(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
    }

    pub fn d_less_one_to_d(&mut self) {
        self.result.push(String::from("D=D-1"));
    }

    pub fn d_plus_a_to_d(&mut self) {
        self.result.push(String::from("D=D+A"));
    }

    pub fn d_plus_a_address_to_d(&mut self) {
        self.result.push(String::from("A=D+A"));
        self.result.push(String::from("D=M"));
    }

    pub fn d_plus_m_to_m(&mut self) {
        self.result.push(String::from("M=D+M"));
    }

    pub fn m_less_d_to_m(&mut self) {
        self.result.push(String::from("M=M-D"));
    }

    pub fn m_less_d_to_d(&mut self) {
        self.result.push(String::from("D=M-D"));
    }

    pub fn m_and_d_to_m(&mut self) {
        self.result.push(String::from("M=M&D"));
    }

    pub fn m_or_d_to_m(&mut self) {
        self.result.push(String::from("M=M|D"));
    }

    pub fn not_m_to_m(&mut self) {
        self.result.push(String::from("M=!M"));
    }

    pub fn neg_m_to_m(&mut self) {
        self.result.push(String::from("M=-M"));
    }

    pub fn compare_with_d(&mut self, compare: &str, label_name: &str) {
        self.result.push(String::from("M=-1")); // m = true
        self.result.push(format!("@{}", label_name)); // if compare is false, set m to false
        self.result.push(format!("D;{}", compare));
        self.result.push(String::from("@SP"));
        self.result.push(String::from("A=M"));
        self.result.push(String::from("M=0"));
        self.result.push(format!("({})", label_name)); // end if
    }

    pub fn m_to_d(&mut self) {
        self.result.push(String::from("D=M"));
    }

    pub fn d_to_m(&mut self) {
        self.result.push(String::from("M=D"));
    }

    pub fn d_to_tmp(&mut self, label: &str) {
        self.result.push(format!("@{}", label));
        self.result.push(String::from("M=D"));
    }

    pub fn move_value_to_d(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("D=A"));
    }

    pub fn get_value_from_segment_plus_d(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("A=M+D"));
        self.result.push(String::from("D=M"));
    }

    pub fn get_address_from_segment_plus_d(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("D=M+D"));
    }

    pub fn get_value_at(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("D=M"));
    }

    pub fn goto_value_at(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("A=M"));
        self.result.push(String::from("0;JMP"));
    }

    pub fn advance_sp(&mut self) {
        self.result.push(String::from("@SP"));
        self.result.push(String::from("M=M+1"));
    }

    pub fn push_to_stack(&mut self) {
        self.result.push(String::from("@SP"));
        self.result.push(String::from("A=M"));
        self.result.push(String::from("M=D"));

        // @SP++
        self.advance_sp();
    }

    pub fn push_to_stack_zero(&mut self) {
        self.result.push(String::from("@SP"));
        self.result.push(String::from("A=M"));
        self.result.push(String::from("M=0"));

        // @SP++
        self.advance_sp();
    }

    pub fn pop_from_stack(&mut self) {
        self.result.push(String::from("@SP"));
        self.result.push(String::from("M=M-1"));
        self.result.push(String::from("A=M"));
    }

    pub fn pop_from_stack_to_d(&mut self) {
        self.pop_from_stack();
        self.result.push(String::from("D=M"));
    }

    pub fn pop_from_stack_to(&mut self, label: &str) {
        self.pop_from_stack_to_d();

        self.result.push(format!("@{}", label));
        self.result.push(String::from("A=M"));
        self.result.push(String::from("M=D"));
    }
}
