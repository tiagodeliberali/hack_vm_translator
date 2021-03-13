use std::collections::HashMap;

pub fn parse_content(content: Vec<String>, filename: String) -> Vec<String> {
    let actions = build_actions();
    let mut result: Vec<String> = Vec::new();

    for line in content {
        result.push(format!("// {}", line));

        let (command, _, _) = split_command(&line);

        let action = actions.get(&command).expect("Invalid action required!");

        result.extend(action(line, filename.as_str()));
    }

    result
}

fn split_command(line: &String) -> (String, String, String) {
    let command: Vec<&str> = line.split(' ').collect();

    if command.len() == 1 {
        return (String::from(command[0]), String::new(), String::new());
    }

    (
        String::from(command[0]),
        String::from(command[1]),
        String::from(command[2]),
    )
}

type Callback = fn(String, &str) -> Vec<String>;

fn build_actions() -> HashMap<String, Callback> {
    let mut actions: HashMap<String, Callback> = HashMap::new();

    actions.insert(String::from("push"), push_action);
    actions.insert(String::from("pop"), pop_action);
    actions.insert(String::from("add"), add_action);
    actions.insert(String::from("sub"), sub_action);

    actions
}

fn push_action(input: String, filename: &str) -> Vec<String> {
    let (_, mem, value) = split_command(&input);
    let mut builder = AssemblerCommandBuilder::new();

    // use value information
    match mem.as_str() {
        "local" => {
            builder.move_value_to_d(value);
            builder.get_value_from_segment("LCL");
            builder.push_to_stack();
        }
        "argument" => {
            builder.move_value_to_d(value);
            builder.get_value_from_segment("ARG");
            builder.push_to_stack();
        }
        "this" => {
            builder.move_value_to_d(value);
            builder.get_value_from_segment("THIS");
            builder.push_to_stack();
        }
        "that" => {
            builder.move_value_to_d(value);
            builder.get_value_from_segment("THAT");
            builder.push_to_stack();
        }
        "constant" => {
            builder.move_value_to_d(value);
            builder.push_to_stack();
        }
        "temp" => {
            builder.move_value_to_d(value);

            builder.at("5");
            builder.d_plus_a_address_to_d();

            builder.push_to_stack();
        }
        "static" => {
            builder.at(format!("{}.{}", filename, value).as_str());
            builder.m_to_d();
            builder.push_to_stack();
        }
        "pointer" => {
            let parsed_value = if value == "0" { "THIS" } else { "THAT" };

            builder.at(format!("{}.{}", filename, parsed_value).as_str());
            builder.m_to_d();
            builder.push_to_stack();
        }
        _ => panic!(format!("Invalid memory location! {}", mem)),
    }

    builder.parsed_content()
}

fn pop_action(input: String, filename: &str) -> Vec<String> {
    let (_, mem, value) = split_command(&input);
    let mut builder = AssemblerCommandBuilder::new();

    // use value information
    match mem.as_str() {
        "local" => {
            builder.move_value_to_d(value);
            builder.get_address_from_segment("LCL");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "argument" => {
            builder.move_value_to_d(value);
            builder.get_address_from_segment("ARG");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "this" => {
            builder.move_value_to_d(value);
            builder.get_address_from_segment("THIS");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "that" => {
            builder.move_value_to_d(value);
            builder.get_address_from_segment("THAT");
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "temp" => {
            builder.move_value_to_d(value);
            builder.at("5");
            builder.d_plus_a_to_d();
            builder.d_to_tmp();
            builder.pop_from_stack_to("tmp");
        }
        "static" => {
            builder.pop_from_stack_to(format!("{}.{}", filename, value).as_str());
        }
        "pointer" => {
            let parsed_value = if value == "0" { "THIS" } else { "THAT" };
            builder.pop_from_stack_to(parsed_value);
        }
        _ => panic!(format!("Invalid memory location! {}", mem)),
    }

    builder.parsed_content()
}

fn add_action(_: String, _: &str) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.d_plus_m_to_m();
    builder.advance_sp();

    builder.parsed_content()
}

fn sub_action(_: String, _: &str) -> Vec<String> {
    let mut builder = AssemblerCommandBuilder::new();

    builder.pop_from_stack_to_d();
    builder.pop_from_stack();
    builder.m_less_d_to_m();
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

    pub fn m_to_d(&mut self) {
        self.result.push(String::from("D=M"));
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
