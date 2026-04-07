use super::*;

fn interaction_setting_text(choice: i32) -> &'static str {
    match choice {
        2 => { "Reverse" },
        3 => { "Self-Interact"},
        1 => { "Random"},
        4 => { "Fates"},
        5 => { "None"},
        6 => { "All"},
        _ => { "Default"},
    }
}
pub fn change_interaction_data(choice: i32, loaded: bool) {
    if loaded && choice == 0 { return; }
    let interact_data = InteractData::get_list_mut().unwrap();
    match choice & 255 {
        2 => {  //Reverse
            let data = get_data_read();
            for x in 0..10 {
                interact_data[x].flag.value = data.interactions[10 + x];
                interact_data[x + 10].flag.value = data.interactions[x];
            }
        }, 
        3 => {  //Self-Interaction
            for x in 0..10 {
                interact_data[x as usize].flag.value =  ( 1 << x ) + ( 1 << (x + 10) );
                interact_data[ x as usize  + 10  ].flag.value =  ( 1 << x ) + ( 1 << (x + 10) );
            } 
        },
        1 => {  // Random 
            if !DVCVariables::random_enabled() { return; }
            let rng = utils::get_rng();
            for x in 0..20 {
                if x % 10 == 0 { continue; }
                let mut chance = 100;
                let mut value: i32 = 0;
                let mut set: [bool; 20] = [false; 20];
                loop {  // for advantages
                    if chance < rng.get_value(100) { break; }
                    let interact = rng.get_value(10);
                    if set[interact as usize] { continue; }
                    value |= 1 << interact;
                    chance = chance / 2;
                    set[interact as usize] = true;
                }
                chance = 100;   
                loop {  // for disadvantage
                    if chance < rng.get_value(100)  { break; }
                    let interact = rng.get_value(10) + 10;
                    if set[interact as usize] { continue; }
                    value |=  1 << interact ;
                    chance = chance / 2;
                    set[interact as usize] = true;
                }
                interact_data[x as usize].flag.value = value;
            }
        },
        4 => {  // Fates Weapon Triangle
            let values = [0, 36888, 24642, 67620, 329764, 286786, 299032, 0, 112, 0, 0, 24612, 67608, 36930, 37186, 67864, 24868, 0, 114688, 0];
            for x in 0..20 { interact_data[x as usize].flag.value = values[x as usize];  }
        },
        5 => { for x in 0..20 { interact_data[x as usize].flag.value = 0; } },
        6 => { 
            for x in 1..10 { 
                interact_data[x as usize].flag.value = -1;
                interact_data[ x as usize + 10].flag.value = -1;
            }
        },
        _ => { GameData::get().reset_interaction(); },
    }
}

