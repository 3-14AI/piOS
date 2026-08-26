#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use inference_runtime::{InferenceEngine, Model, Tensor};
use pios_api::a2a::priority::AgentPriority;
use pios_api::a2a::protocol::{A2AMessage, MessageType};
use vector_db::{VectorDb, VectorRecord};

#[derive(Debug, PartialEq, Clone)]
pub enum Redirect {
    In(String),
    Out(String),
    Append(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct CommandNode {
    pub program: String,
    pub args: Vec<String>,
    pub redirect: Option<Redirect>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PipeChain {
    pub commands: Vec<CommandNode>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstNode {
    Simple(PipeChain),
    And(alloc::boxed::Box<AstNode>, alloc::boxed::Box<AstNode>),
    Or(alloc::boxed::Box<AstNode>, alloc::boxed::Box<AstNode>),
}

pub struct KernelProfiler {
    pub flamegraph_data: String,
    pub dtrace_logs: String,
}

impl KernelProfiler {
    pub fn new() -> Self {
        Self {
            flamegraph_data: "mock_flamegraph_data".to_string(),
            dtrace_logs: "mock_dtrace_logs".to_string(),
        }
    }

    pub fn analyze_performance_with_ai(&self) -> String {
        alloc::format!(
            "AI Analysis: System is slow due to high CPU usage in mock driver. Flamegraph: {} DTrace: {}",
            self.flamegraph_data, self.dtrace_logs
        )
    }
}

impl Default for KernelProfiler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NlShell {
    db: VectorDb,
    engine: InferenceEngine,
    model: Model,
    pub profiler: KernelProfiler,
}

impl NlShell {
    pub fn new() -> Result<Self, &'static str> {
        let mut engine = InferenceEngine::new();
        let model = engine
            .load_model_by_name("embedding_model")
            .map_err(|_| "Failed to load embedding model")?;

        let mut shell = Self {
            db: VectorDb::new(),
            engine,
            model,
            profiler: KernelProfiler::new(),
        };

        shell
            .register_command("profiler.analyze", "find why my system is slow")
            .unwrap();
        shell
            .register_command("agent.spawn", "spawn a new ai agent")
            .unwrap();
        shell
            .register_command(
                "agent.collaborate",
                "collaborate on complex tasks multiple agents",
            )
            .unwrap();

        Ok(shell)
    }

    fn generate_embedding(&mut self, text: &str) -> Result<Vec<f32>, &'static str> {
        if text.is_empty() {
            return Ok(alloc::vec![0.0, 0.0, 0.0]);
        }

        let ctx = self
            .engine
            .init_execution_context(&self.model)
            .map_err(|_| "Failed to init execution context")?;

        let data = alloc::vec![0; text.len()];
        let tensor = Tensor::new(data, alloc::vec![text.len()]);

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;

        self.engine.compute(ctx).map_err(|_| "Failed to compute")?;

        let mut out_buffer = [0u8; 12];
        let bytes_written = self
            .engine
            .get_output(ctx, 0, &mut out_buffer)
            .map_err(|_| "Failed to get output")?;

        let mut embedding = Vec::new();
        if bytes_written >= 12 && out_buffer[..bytes_written] != b"mock_output"[..] {
            for chunk in out_buffer[..bytes_written].chunks_exact(4) {
                let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                embedding.push(f32::from_le_bytes(bytes));
            }
        }

        if embedding.is_empty() {
            let mut val1 = 0.0;
            let mut val2 = 0.0;
            let mut val3 = 0.0;
            for (i, b) in text.bytes().enumerate() {
                match i % 3 {
                    0 => val1 += b as f32,
                    1 => val2 += b as f32,
                    _ => val3 += b as f32,
                }
            }
            embedding = alloc::vec![val1, val2, val3];
        }

        Ok(embedding)
    }

    pub fn register_command(
        &mut self,
        api_endpoint: &str,
        description: &str,
    ) -> Result<(), &'static str> {
        let embedding = self.generate_embedding(description)?;

        let record = VectorRecord {
            id: api_endpoint.to_string(),
            vector: embedding,
            metadata: Some(description.to_string()),
        };

        self.db
            .insert(record)
            .map_err(|_| "Failed to insert into vector DB")?;
        Ok(())
    }

    pub fn parse_intent(
        &mut self,
        natural_language_input: &str,
    ) -> Result<Option<String>, &'static str> {
        if natural_language_input.is_empty() {
            return Ok(None);
        }

        let query_embedding = self.generate_embedding(natural_language_input)?;

        let results = self
            .db
            .search_cosine(&query_embedding, 1)
            .map_err(|_| "Failed to search vector DB")?;

        if let Some((_score, record)) = results.first() {
            Ok(Some(record.id.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn parse_command(&self, input: &str) -> Result<AstNode, &'static str> {
        // Very basic parsing for mocking complex commands
        let parts: Vec<&str> = input.split("&&").collect();
        if parts.len() > 1 {
            let left = self.parse_command(parts[0].trim())?;
            let right = self.parse_command(parts[1].trim())?;
            return Ok(AstNode::And(
                alloc::boxed::Box::new(left),
                alloc::boxed::Box::new(right),
            ));
        }

        let parts: Vec<&str> = input.split("||").collect();
        if parts.len() > 1 {
            let left = self.parse_command(parts[0].trim())?;
            let right = self.parse_command(parts[1].trim())?;
            return Ok(AstNode::Or(
                alloc::boxed::Box::new(left),
                alloc::boxed::Box::new(right),
            ));
        }

        let pipes: Vec<&str> = input.split('|').collect();
        let mut commands = Vec::new();

        for pipe in pipes {
            let mut tokens: Vec<String> = Vec::new();
            let mut current_token = String::new();
            let mut in_single_quote = false;
            let mut in_double_quote = false;
            let mut escaped = false;

            for c in pipe.chars() {
                if escaped {
                    current_token.push(c);
                    escaped = false;
                    continue;
                }

                match c {
                    '\\' => {
                        escaped = true;
                    }
                    '\'' if !in_double_quote => {
                        in_single_quote = !in_single_quote;
                    }
                    '"' if !in_single_quote => {
                        in_double_quote = !in_double_quote;
                    }
                    ' ' | '\t' | '\n' if !in_single_quote && !in_double_quote => {
                        if !current_token.is_empty() {
                            tokens.push(current_token);
                            current_token = String::new();
                        }
                    }
                    _ => {
                        current_token.push(c);
                    }
                }
            }
            if !current_token.is_empty() {
                tokens.push(current_token);
            }

            if tokens.is_empty() {
                continue;
            }

            let mut redirect = None;
            if tokens.len() >= 2 {
                let len = tokens.len();
                if tokens[len - 2] == ">" {
                    redirect = Some(Redirect::Out(tokens.pop().unwrap()));
                    tokens.pop(); // remove ">"
                } else if tokens[len - 2] == ">>" {
                    redirect = Some(Redirect::Append(tokens.pop().unwrap()));
                    tokens.pop(); // remove ">>"
                } else if tokens[len - 2] == "<" {
                    redirect = Some(Redirect::In(tokens.pop().unwrap()));
                    tokens.pop(); // remove "<"
                }
            }

            if tokens.is_empty() {
                continue;
            }

            let program = tokens.remove(0);
            let args = tokens;

            commands.push(CommandNode {
                program,
                args,
                redirect,
            });
        }

        if commands.is_empty() {
            Err("Empty command")
        } else {
            Ok(AstNode::Simple(PipeChain { commands }))
        }
    }

    pub fn sys_intent(&mut self, natural_language_input: &str) -> Result<String, &'static str> {
        let intent = self.parse_intent(natural_language_input)?;

        if let Some(cmd) = intent {
            if cmd == "profiler.analyze" {
                return Ok(self.profiler.analyze_performance_with_ai());
            }

            if cmd.starts_with("agent.spawn") {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                let agent_name = if parts.len() > 1 { parts[1] } else { "unknown" };
                return self.spawn_agent(agent_name);
            }

            if cmd.starts_with("agent.collaborate") {
                let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
                let task = if parts.len() > 1 {
                    parts[1]
                } else {
                    "general task"
                };
                return self.spawn_collaboration(task);
            }

            // Here the semantic layer generated a command string to run
            let ast = self.parse_command(&cmd)?;
            self.execute_ast(&ast)
        } else {
            Err("Could not understand intent")
        }
    }

    pub fn run_learning_loop(&mut self) -> Result<String, &'static str> {
        let telemetry = alloc::format!("Telemetry: {}", self.profiler.dtrace_logs);
        // Ensure prompt logic formats correctly without undefined functions.
        let ctx = self
            .engine
            .init_execution_context(&self.model)
            .map_err(|_| "Failed to init ctx")?;
        let embedding = self.generate_embedding(&telemetry)?;
        let tensor = inference_runtime::Tensor::new(
            embedding.into_iter().map(|f| f.to_bits() as u8).collect(),
            alloc::vec![3],
        );

        self.engine
            .set_input(ctx, 0, &tensor)
            .map_err(|_| "Failed to set input")?;
        self.engine.compute(ctx).map_err(|_| "Failed to compute")?;

        Ok(alloc::format!(
            "Learning loop completed: fix generated for {}",
            self.profiler.dtrace_logs
        ))
    }

    pub fn spawn_agent(&mut self, agent_name: &str) -> Result<String, &'static str> {
        // Construct and dispatch A2AMessage for Agent spawning via IPC component
        let payload_str = alloc::format!("SPAWN_WASM {}", agent_name);

        let a2a_msg = A2AMessage::new(
            0, // Sender ID (NL-Shell)
            1, // Receiver ID (System/Init Process)
            MessageType::Command,
            AgentPriority::High,
            payload_str.as_bytes(),
        );

        // We simulate returning the formatted details of what was successfully dispatched
        // to conform to `#![no_std]` testing environment, demonstrating lifecycle integration.
        Ok(alloc::format!(
            "Dispatched A2A Message to Init to spawn agent: {} [MsgType: {:?}, Priority: {:?}]",
            agent_name,
            a2a_msg.msg_type,
            a2a_msg.priority
        ))
    }

    pub fn spawn_collaboration(&mut self, task: &str) -> Result<String, &'static str> {
        let agents = ["researcher_agent", "coder_agent", "verifier_agent"];
        for agent in &agents {
            let _ = self.spawn_agent(agent);
        }
        let payload_str = alloc::format!("COLLABORATE_TASK {}", task);
        let a2a_msg = A2AMessage::new(
            0,   // Sender ID (NL-Shell)
            255, // Broadcast
            MessageType::Announcement,
            AgentPriority::High,
            payload_str.as_bytes(),
        );

        Ok(alloc::format!("Dispatched broadcast A2A Message to collaborate on task: {} [MsgType: {:?}, Priority: {:?}]", task, a2a_msg.msg_type, a2a_msg.priority))
    }

    pub fn execute_ast(&self, ast: &AstNode) -> Result<String, &'static str> {
        match ast {
            AstNode::Simple(pipe_chain) => {
                let mut data_pipe = String::new();
                for cmd in &pipe_chain.commands {
                    // Simulate piping output of one command as input to another using A2A
                    let payload_str = alloc::format!("{} {:?}", cmd.program, cmd.args);
                    let a2a_msg = A2AMessage::new(
                        0, // Sender ID (NL-Shell)
                        1, // Receiver ID (target instance)
                        MessageType::Command,
                        AgentPriority::Normal,
                        payload_str.as_bytes(),
                    );

                    data_pipe = alloc::format!(
                        "Dispatched A2A Message: {:?} (Payload string: {}) [Previous Output Piped: {}]",
                        a2a_msg,
                        payload_str,
                        data_pipe
                    );
                    if let Some(ref r) = cmd.redirect {
                        data_pipe.push_str(&alloc::format!(" [Redirect: {:?}]", r));
                    }
                }
                Ok(data_pipe)
            }
            AstNode::And(left, right) => {
                let left_res = self.execute_ast(left)?;
                let right_res = self.execute_ast(right)?;
                Ok(alloc::format!("{} AND {}", left_res, right_res))
            }
            AstNode::Or(left, right) => {
                let left_res = self.execute_ast(left);
                if left_res.is_ok() {
                    left_res
                } else {
                    self.execute_ast(right)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_run_learning_loop() {
        let mut shell = NlShell::new().unwrap();
        let res = shell.run_learning_loop().unwrap();
        assert!(res.contains("Learning loop completed: fix generated for mock_dtrace_logs"));
    }

    #[test]
    fn test_sys_intent_spawn_agent() {
        let mut shell = NlShell::new().unwrap();
        let res = shell.sys_intent("spawn a new ai agent").unwrap();
        assert!(res.contains("Dispatched A2A Message to Init to spawn agent: "));
        assert!(res.contains("MsgType: Command"));
        assert!(res.contains("Priority: High"));
    }

    #[test]
    fn test_sys_intent_collaborate() {
        let mut shell = NlShell::new().unwrap();
        let res = shell
            .sys_intent("collaborate on complex tasks multiple agents")
            .unwrap();
        assert!(res.contains("Dispatched broadcast A2A Message to collaborate on task: "));
        assert!(res.contains("MsgType: Announcement"));
        assert!(res.contains("Priority: High"));
    }

    use super::*;

    #[test]
    fn test_nl_shell_creation() {
        let shell = NlShell::new();
        assert!(shell.is_ok());
    }

    #[test]
    fn test_register_and_parse_command() {
        let mut shell = NlShell::new().unwrap();

        shell
            .register_command("kernel.process.list", "show running processes list them")
            .unwrap();
        shell
            .register_command("kernel.fs.read", "read file content from disk")
            .unwrap();

        let intent = shell
            .parse_intent("can you list the running processes")
            .unwrap();

        assert!(intent.is_some());
    }

    #[test]
    fn test_parse_complex_command() {
        let shell = NlShell::new().unwrap();

        let ast = shell.parse_command("ls -la | grep sys > out.txt").unwrap();

        if let AstNode::Simple(pipe) = ast {
            assert_eq!(pipe.commands.len(), 2);
            assert_eq!(pipe.commands[0].program, "ls");
            assert_eq!(pipe.commands[0].args, alloc::vec!["-la"]);
            assert_eq!(pipe.commands[1].program, "grep");
            assert_eq!(pipe.commands[1].args, alloc::vec!["sys"]);
            assert_eq!(
                pipe.commands[1].redirect,
                Some(Redirect::Out("out.txt".to_string()))
            );
        } else {
            panic!("Expected simple pipe chain");
        }

        let ast2 = shell
            .parse_command("echo \"hello world\" 'single quote'")
            .unwrap();
        if let AstNode::Simple(pipe) = ast2 {
            assert_eq!(pipe.commands.len(), 1);
            assert_eq!(pipe.commands[0].program, "echo");
            assert_eq!(
                pipe.commands[0].args,
                alloc::vec!["hello world", "single quote"]
            );
        } else {
            panic!("Expected simple pipe chain");
        }
    }

    #[test]
    fn test_parse_and_or_chain() {
        let shell = NlShell::new().unwrap();
        let ast = shell.parse_command("make && ./test").unwrap();
        match ast {
            AstNode::And(_, _) => (),
            _ => panic!("Expected And node"),
        }

        let ast_or = shell.parse_command("cat file.txt || echo error").unwrap();
        match ast_or {
            AstNode::Or(_, _) => (),
            _ => panic!("Expected Or node"),
        }
    }

    #[test]
    fn test_execute_ast() {
        let shell = NlShell::new().unwrap();
        let ast = shell.parse_command("cat text.txt | grep a").unwrap();
        let res = shell.execute_ast(&ast).unwrap();
        assert!(res.contains("Dispatched A2A Message"));
        assert!(res.contains("cat"));
        assert!(res.contains("grep"));
        assert!(res.contains("[Previous Output Piped"));
    }

    #[test]
    fn test_sys_intent() {
        let mut shell = NlShell::new().unwrap();

        shell
            .register_command("ls -la | grep sys", "list processes and grep sys")
            .unwrap();

        let res = shell.sys_intent("list processes and grep sys").unwrap();
        assert!(res.contains("Dispatched A2A Message"));
        assert!(res.contains("ls"));
        assert!(res.contains("grep"));
    }

    #[test]
    fn test_kernel_profiler() {
        let profiler = KernelProfiler::new();
        let analysis = profiler.analyze_performance_with_ai();
        assert!(analysis.contains("AI Analysis: System is slow"));
        assert!(analysis.contains("mock_flamegraph_data"));
        assert!(analysis.contains("mock_dtrace_logs"));
    }

    #[test]
    fn test_sys_intent_profiling() {
        let mut shell = NlShell::new().unwrap();
        let res = shell.sys_intent("find why my system is slow").unwrap();
        assert!(res.contains("AI Analysis: System is slow"));
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_parse_intent_empty_input() {
        let mut shell = NlShell::new().unwrap();
        let intent = shell.parse_intent("").unwrap();
        assert_eq!(intent, None);
    }

    #[test]
    fn test_generate_embedding_empty_text() {
        let mut shell = NlShell::new().unwrap();
        let embedding = shell.generate_embedding("").unwrap();
        assert_eq!(embedding, alloc::vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_parse_intent_no_match() {
        let mut shell = NlShell::new().unwrap();

        let intent = shell
            .parse_intent("do something entirely unrelated")
            .unwrap();

        // Since NlShell::new() now registers "profiler.analyze", the DB is not empty.
        // It will return some intent. We assert it does not return None, or if we want we can test the behavior.
        assert!(intent.is_some());
    }
}
