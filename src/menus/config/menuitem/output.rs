use std::{io::Write, path::Path};
use std::fs::File;
use super::*;
use crate::gamedata::*;
use crate::{VARIABLE_VERSION, VERSION};
use crate::utils::to_opt;

pub fn export_result() ->  std::io::Result<()> {
    let hero =  UnitPool::get_hero(false);
    let path = format!("sd:/DVC/{}.txt", hero.get_person().get_ascii_name());
    let mut file = File::options().create(true).write(true).truncate(true).open(path.as_str())?;
    writeln!(&mut file, "Draconic Vibe Crystal v. {}", VERSION)?;
    writeln!(&mut file, "* Variable Version: {}", VARIABLE_VERSION)?;
    writeln!(&mut file, "* DVC Seed: {}", DVCVariable::Seed.get_setting_value() as u32)?;
    writeln!(&mut file, "* Alear Gender: {}", if DVCVariables::is_lueur_female() { "Female" } else { "Male" })?;
    writeln!(file, "\n*** Randomized Person Data ***")?;
    let gamedata = GameData::get();
    let playables = PersonPool::get();
    if DVCVariable::UnitRecruitment.get_value() > 0 { writeln!(&mut file, "* Unit Recruitment Mode: {}", DVCVariable::UnitRecruitment.get_value())?; }
    playables.playable.iter().enumerate().for_each(|(i, p)|{
        let data = p.get_person_data();
        let _ = writeln!(file, "{}: {} / {} ", i+1, engage::app::Mess::get(data.get_name()), data.get_pid());
        if let Some(skill) = to_opt(SkillData::try_get_from_hash(GameVariableManager::get_number(format!("G_P_{}", data.get_pid())))){
            let _ = writeln!(file, "\tSkill: {}", engage::app::Mess::get(skill.get_name()));
        }
        if let Some(job) = to_opt(JobData::try_get_from_hash(GameVariableManager::get_number(format!("G_JG_{}", data.get_pid())))){
            let _ = writeln!(file, "\tStarting Class: {}", engage::app::Mess::get(job.get_name()));
        }
        if let Some(r) = p.get_replacement(false).filter(|v| v.hash() != data.hash()) {
            let _ = writeln!(file, "\tReplaced by {} / {}", engage::app::Mess::get(r.get_name()), r.get_pid());
        }
    });
    let grow_mode =
        match DVCVariable::PersonalGrowthMode.get_setting_value() {
            1 => { "Restrict (R)" }
            2 => { "Unrestrict (U)" }
            3 => { "Shuffled (S)" }
            4 => { "Chaos (C)" }
            _ => { "Normal" }
        };
    writeln!(file, "\n*** Personal Growth Mode: {} ***", grow_mode)?;

    writeln!(file, "* Modes: Restrict (R), Unrestrict (U), Shuffled (S), Chaos (C)")?;
    if DVCFlags::AdaptiveGrowths.get_setting_value() { writeln!(file, "* Growths are shown without 'Adaptive Growths' adjustment")?; }
    writeln!(file, "* Mode | HP | Str | Mag | Dex | Spd | Lck | Def | Res | Bld |\n")?;
    playables.playable.iter().enumerate().for_each(|(i, p)|{
        let data = p.get_person_data();
        let _ = writeln!(file, "{} / {} ", engage::app::Mess::get(data.get_name()), data.get_pid());
        ["R", "U", "S", "C"].into_iter().enumerate().for_each(|(i, v)|{
            let grow = gamedata.growth_data.calculate_growth(p.hash, i as i32 + 1);
            let mut growth_str = number_with_space(grow[0]);
            [1, 6, 2, 3, 4, 5, 7, 8].into_iter().for_each(|i|{
                growth_str += " | ";
                growth_str += number_with_space(grow[i]).as_str();
            });
            growth_str += " |";
            let _ = writeln!(file, "\t{}: {}", v, growth_str.as_str());
        });
    });
    Ok(())
}
fn number_with_space(v: u8) -> String {
    if v < 10 { format!("  {}", v) }
    else if v < 100 { format!(" {}", v) }
    else { format!("{}", v) }
}