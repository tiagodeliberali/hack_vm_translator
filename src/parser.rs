use rand::prelude::*;
use std::collections::HashMap;

pub fn parse_content(content: Vec<String>, filename: String) -> Vec<String> {
    let actions = build_actions();
    let mut result: Vec<String> = Vec::new();

    for line in content {
        result.push(format!("// {}", line));

        let instruction = VMInstruction::new(&line, &filename);

        let action = actions
            .get(&instruction.command)
            .expect("Invalid action required!");

        result.extend(action(instruction));
    }

    result
}

struct VMInstruction {
    pub command: String,
    pub detail: String,
    pub value: String,
    pub filename: String,
}

impl VMInstruction {
    pub fn new(line: &String, filename: &str) -> VMInstruction {
        let (command, detail, value) = VMInstruction::split_command(&line);

        VMInstruction {
            command,
            detail,
            value,
            filename: String::from(filename),
        }
    }

    fn split_command(line: &String) -> (String, String, String) {
        let command: Vec<&str> = line.split(' ').collect();

        if command.len() == 1 {
            return (String::from(command[0]), String::new(), String::new());
        }

        if command.len() == 2 {
            return (
                String::from(command[0]),
                String::from(command[1]),
                String::new(),
            );
        }

        (
            String::from(command[0]),
            String::from(command[1]),
            String::from(command[2]),
        )
    }
}

type Callback = fn(VMInstruction) -> Vec<String>;

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

    actions
}

fn push_action(instruction: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    // use value information
    match instruction.detail.as_str() {
        "local" => {
            builder.move_value_to_d(instruction.value);
            builder.get_value_from_segment("LCL");
            builder.push_to_stack();
        }
        "argument" => {
            builder.move_value_to_d(instruction.value);
            builder.get_value_from_segment("ARG");
            builder.push_to_stack();
        }
        "this" => {
            builder.move_value_to_d(instruction.value);
            builder.get_value_from_segment("THIS");
            builder.push_to_stack();
        }
        "that" => {
            builder.move_value_to_d(instruction.value);
            builder.get_value_from_segment("THAT");
            builder.push_to_stack();
        }
        "constant" => {
            builder.move_value_to_d(instruction.value);
            builder.push_to_stack();
        }
        "temp" => {
            builder.move_value_to_d(instruction.value);

            builder.at("5");
            builder.d_plus_a_address_to_d();

            builder.push_to_stack();
        }
        "static" => {
            builder.at(format!("{}.{}", instruction.filename, instruction.value).as_str());
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

fn pop_action(instruction: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    // use value information
    match instruction.detail.as_str() {
        "local" => {
            builder.move_value_to_d(instruction.value);
            builder.get_address_from_segment("LCL");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "argument" => {
            builder.move_value_to_d(instruction.value);
            builder.get_address_from_segment("ARG");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "this" => {
            builder.move_value_to_d(instruction.value);
            builder.get_address_from_segment("THIS");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "that" => {
            builder.move_value_to_d(instruction.value);
            builder.get_address_from_segment("THAT");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "temp" => {
            builder.move_value_to_d(instruction.value);
            builder.at("5");
            builder.d_plus_a_to_d();
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "static" => {
            let parsed_value = format!("{}.{}", instruction.filename, instruction.value);

            builder.pop_from_stack_to_d();
            builder.at(parsed_value.as_str());
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

fn add_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.d_plus_m_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn sub_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn eq_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_d();
    builder.compare_with_d("JEQ");
    builder.advance_sp();

    builder.parsed_content()
}

fn lt_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_d();
    builder.compare_with_d("JLT");
    builder.advance_sp();

    builder.parsed_content()
}

fn gt_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_d();
    builder.compare_with_d("JGT");
    builder.advance_sp();

    builder.parsed_content()
}

fn and_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_and_d_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn or_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_or_d_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn not_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack();
    builder.not_m_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn neg_action(_: VMInstruction) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack();
    builder.neg_m_to_m();
    builder.advance_sp();

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

    pub fn at(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
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

    pub fn compare_with_d(&mut self, compare: &str) {
        let random_jump: u32 = rand::thread_rng().gen();
        let jump_name = format!("FALSE.{}", random_jump);

        self.result.push(String::from("M=-1")); // m = true
        self.result.push(format!("@{}", jump_name)); // if compare is false, set m to false
        self.result.push(format!("D;{}", compare));
        self.result.push(String::from("@SP"));
        self.result.push(String::from("A=M"));
        self.result.push(String::from("M=0"));
        self.result.push(format!("({})", jump_name)); // end if
    }

    pub fn m_to_d(&mut self) {
        self.result.push(String::from("D=M"));
    }

    pub fn d_to_m(&mut self) {
        self.result.push(String::from("M=D"));
    }

    pub fn d_to_tmp(&mut self) {
        self.result.push(String::from("@tmp"));
        self.result.push(String::from("M=D"));
    }

    pub fn move_value_to_d(&mut self, value: String) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("D=A"));
    }

    pub fn get_value_from_segment(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("A=M+D"));
        self.result.push(String::from("D=M"));
    }

    pub fn get_address_from_segment(&mut self, value: &str) {
        self.result.push(format!("@{}", value));
        self.result.push(String::from("D=M+D"));
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
