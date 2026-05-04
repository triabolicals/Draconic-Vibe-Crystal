use engage::{
    random::Random,
    gamedata::{Gamedata, GodData, skill::SkillData}
};
use skyline::patching::Patch;
use crate::{config::{DVCFlags, DVCVariables}, randomizer::data::{EngageAtk, GameData}, randomizer::Randomizer};

#[derive(Clone)]
pub struct EngageAttackRandomizer {
    pub atks: Vec<EngageAtk>,
    pub weapon_prohibit: Vec<i32>,
}

impl EngageAttackRandomizer {
    pub fn new(n_emblems: usize) -> Self { Self { atks: vec![EngageAtk::default(); n_emblems], weapon_prohibit: vec![0; n_emblems], } }
    pub fn randomize(&mut self, data: &GameData) {
        let emblem_list: Vec<_> = data.emblem_pool.emblem_list.iter().enumerate().filter(|x|  x.0 < 20 || x.0 >= 24).map(|(_, x)| *x).collect();
        let rng = Random::new(3*DVCVariables::get_seed() as u32);
        let engage_atks = &data.skill_pool.engage_attacks;
        let mut engage_atk_pool: Vec<_> = engage_atks.iter().collect();
        let mut available_emblem_list: Vec<usize> = (0..emblem_list.len()).collect();
        let mut linked_engage_atk: Vec<i32> = engage_atks.iter().map(|x| x.0 ).collect();
        let astra_storm: Vec<_> = engage_atks.iter().filter(|s| SkillData::try_get_hash(s.0).is_some_and(|s| s.sid.str_contains("_リン"))).map(|x| x.0).collect();

        let n_emblems = available_emblem_list.len();
        available_emblem_list.remove(19);   // No Emblem Alear for Engage+ Links
        for x in 0..n_emblems {
            if x == 19 { continue; }
            let pool_size = available_emblem_list.len() as i32;
            if pool_size > 1 {
                let mut index = rng.get_value(pool_size);
                let mut count = 0;
                while count < 10 && x == available_emblem_list[index as usize] {
                    count += 1;
                    index = rng.get_value(pool_size);
                }
                self.atks[x].linked_emblem = available_emblem_list[index as usize] as i32;
                available_emblem_list.remove(index as usize);
            }
            else {
                self.atks[x].linked_emblem = available_emblem_list[0] as i32;
                available_emblem_list.remove(0);
                break;
            }
        }
        for x in 0..n_emblems {
            let size = engage_atk_pool.len();
            if size > 1 {
                let mut selection = rng.get_value( engage_atk_pool.len() as i32) as usize;
                if x == 9 { // Byleth gets no Astra Storm
                    loop {
                        selection = rng.get_value( engage_atk_pool.len() as i32) as usize;
                        if !astra_storm.contains( &engage_atk_pool[selection].0 ) {
                            break;
                        }
                    }
                }
                if let Some(z) = engage_atk_pool.get_remove(rng) { self.atks[x].engage_atk = z.0; }
                if let Some(z) = linked_engage_atk.get_remove(rng) { self.atks[x].linked_engage_atk = z; }
            }
        }
    }
    pub fn commit(&self, data: &GameData) {
        let emblem_list = GameData::get_playable_god_list();
        if DVCFlags::EngageAttacks.get_value() {
            let names: Vec<_> = emblem_list.iter().map(|g| g.gid).collect();
            Patch::in_text(0x01c77620).bytes(&[0xc0, 0x03, 0x5f, 0xd6]).unwrap();
            emblem_list.iter().zip(self.atks.iter()).for_each(|(x, y)| {
                if let Some(engage_atk) = SkillData::try_get_hash(y.engage_atk) {
                    x.set_engage_attack(Some(engage_atk.sid));
                    x.change_data.iter().for_each(|g| g.set_engage_attack(Some(engage_atk.sid)));
                    if let Some(opp_god) = GodData::get_mut(x.gid.to_string().replace("GID_", "GID_相手")) {
                        opp_god.set_engage_attack(Some(engage_atk.sid));
                        opp_god.change_data.iter().for_each(|g| { g.set_engage_attack(Some(engage_atk.sid)); });
                    }
                }
                if let Some(linked_atk) = SkillData::try_get_hash(y.linked_engage_atk) { x.change_data.iter().for_each(|g| g.set_engage_attack_link(linked_atk.sid)); }
                if y.linked_emblem >= 0 && y.linked_emblem < emblem_list.len() as i32 { x.change_data.iter().for_each(|g| { g.set_link_gid(names[y.linked_emblem as usize]); }); }
            });
            data.emblem_pool.emblem_persons.iter().filter(|x| x.engage_atk != 0).map(|p| (p.emblem_index, p.get_person(), p.is_paralogue() && p.is_custom()))
                .for_each(|(i, p, custom)| {
                    if custom {
                        let engage_atk = DVCVariables::get_god_from_index(i as i32, true)
                            .and_then(|g| g.engage_attack.and_then(|sid| SkillData::get(sid)));
                        
                        p.set_engage_skill(engage_atk);
                    }
                    else if let Some(engage) = self.atks.get(i) { 
                        p.set_engage_skill(SkillData::try_index_get(engage.engage_atk)); 
                    }
                });
        }
        else {
            data.emblem_pool.emblem_persons.iter().for_each(|p|{ p.reset_engage_skill(data);});
            data.emblem_pool.emblem_data.iter().for_each(|x|{
                if let Some(god) = GodData::try_get_hash_mut(x.hash) {
                    if let Some(engage_atk) = SkillData::try_get_hash(x.engage_atk) {
                        god.change_data.iter().for_each(|g| g.set_engage_attack(Some(engage_atk.sid)));
                        if let Some(opp_god) = GodData::get_mut( god.gid.to_string().replace("GID_", "GID_相手")) {
                            opp_god.change_data.iter().for_each(|g|{ g.set_engage_attack(Some(engage_atk.sid)); });
                        }
                    }
                    if let Some(linked_atk) = SkillData::try_get_hash(x.link_engage_atk) {
                        god.change_data.iter().for_each(|g| g.set_engage_attack_link(linked_atk.sid));
                    }
                    god.change_data.iter_mut().for_each(|g|{ g.link_gid = x.link_gid.as_ref().map(|str| str.into()); });
                }
            });
        }
    }
}