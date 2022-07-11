use crate::data::{ServerCommand, ClientCommand};
use serde_json::{to_value, from_value, from_str, to_writer, Value};
use serde::{Serialize, Deserialize};
use std::io::Write as _;

// For full ser/de of commands
// ClientCommand can be serialized,
// ServerCommand can be deserialized
// But mutually they can suck one.

#[derive(Serialize, Deserialize)]
struct CommandDummy {
    command: String, // This was originally &'a str, but this caused serde to complain about Deserialize having insufficient lifetimes.
    data: Value
}

// Use Serde to convert them to/from adjacent format
fn parse_command(command: &str) -> ServerCommand {
    // Split the command into the JSON data body and the command head
    let (head, data) = command.split_at(4);
    from_value(to_value(CommandDummy {
        command: head.trim().to_string(),
        data: from_str(data).expect("Unable to parse data to Value")
    }).expect("Unable to convert CommandDummy to Value"))
    .expect("Unable to convert Value to ServerCommand") // Forgive me, for I have sinned.
}

fn prepare_command(command: &ClientCommand) -> String {
    let dummy_value: CommandDummy = from_value(to_value(command).expect("Unable to convert command to Value")).expect("Unable to convert Value to CommandDummy");
    let mut command_buffer = Vec::with_capacity(256);
    write!(&mut command_buffer, "{} ", dummy_value.command).expect("Failed to write command head");
    to_writer(&mut command_buffer, &dummy_value.data).expect("Failed to write command data-body");
    String::from_utf8_lossy(&command_buffer).to_string()
}