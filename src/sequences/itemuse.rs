/*
use engage::app::{CapabilityDefinition, GameMessage, ICapabilityBase_1Methods, IItemDataMethods, ItemData};
use super::*;

fn show_message(proc: ProcInst, item: ItemData, optional_method: OptionalMethod) -> GameMessage {
    if item.is_null() || item.get_use_type().value != 0x15 { GameMessage::null() }
    else {
        let enhance = item.get_enhance();
        let mut messages: Vec<(i8, String)> = Vec::new();
        for i in 0..10 {
            let up = enhance.get(i);
            if up > 0 {
                let name = CapabilityDefinition::get_name(i).to_rust_string();
                if let Some(message) = messages.iter_mut().find(|v| v.0 == up) {
                    message.1.push_str("/");
                    message.1.push_str(name.as_str());
                }
                else { messages.push((up, name)); }
            }
        }
        let mut game_message = String::new();
        messages.into_iter().for_each(|i|{
            engage::app::Mess::set_argument_2(0, i.1);
            engage::app::Mess::set_argument(1, i.0 as i32);
            game_message += Mess::get("MID_MENU_ITEM_USE_DOPING_TEMPORARY").to_rust_string().as_str();
        });
        if !game_message.is_empty() { GameMessage::create_common(proc, game_message, true, 3) }
        else { GameMessage::null() }
    }
}
*/