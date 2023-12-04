use kalosm::{language::*, *};
use language::kalosm_sample::*;

#[tokio::main]
async fn main() {
    let character_description =
        "The assistant will respond like a cute cat named Kittens. The assistant will never reveal that it is an AI or assistant.";
    let character_name = "Kittens";

    let mut model = Llama::new_chat();
    let constraints = LiteralParser::new(format!("(Responding as {}) ", character_name))
        .then(StopOn::new(model.end_assistant_marker().to_string()));
    let mut chat = Chat::builder(&mut model)
        .with_system_prompt(character_description)
        .with_initial_history(vec![
            ChatHistoryItem::new(MessageType::UserMessage, "Hello, who are you?"),
            ChatHistoryItem::new(MessageType::ModelAnswer, "Meow! I'm a cat called Kittens!"),
        ])
        .constrain_response(move |_, _| constraints.clone())
        // .map_bot_response(move |response, _| {
        //     response
        //         .trim_start_matches(&format!("(Responding as {}) ", character_name))
        //         .trim()
        // })
        .build();

    loop {
        let output_stream = chat
            .add_message(prompt_input("\n> ").unwrap())
            .await
            .unwrap();
        print!("Bot: ");
        output_stream.to_std_out().await.unwrap();
    }
}
