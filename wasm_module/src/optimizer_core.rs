use wasm_bindgen::JsValue;
use web_sys::console;

use crate::{
    gw2data::{Affix, Attribute, AttributesArray, Character, Combination},
    result::Result,
    utils::round_even,
    utils::{clamp, print_attr},
};
use std::cell::RefCell;

/// Uses depth-first search to calculate all possible combinations of affixes for the given subtree.
///
/// # Arguments
/// * `affix_array` - An array of vectors of affixes. Each entry in the array corresponds to the affixes selectable for a specific slot. The array is of length 14, because there are 14 slots. However, if the last slot is not used due to two-handed weapons, the last entry in the array is Affix::None
/// * `subtree` - The current subtree of the affix tree. This is a vector of affixes. The length of the vector is the current layer of the tree. The first entry in the vector is the root of the tree.
/// * `leaf_callback` - A function that is called when a leaf of the tree is reached. The function is passed the current subtree.
pub fn descend_subtree_dfs<F>(
    affix_array: &[Vec<Affix>],
    subtree: &[Affix],
    max_depth: usize,
    leaf_callback: &mut F,
) where
    F: FnMut(&[Affix]),
{
    let current_layer = subtree.len();

    if current_layer == max_depth {
        // if we reached leafs of the tree, call the function
        leaf_callback(subtree);
    } else {
        let permutation_options = &affix_array[current_layer];

        let mut new_subtree: Vec<Affix> = Vec::with_capacity(subtree.len() + 1);
        new_subtree.clear();
        new_subtree.extend_from_slice(subtree);

        for &option in permutation_options {
            new_subtree.push(option);
            descend_subtree_dfs(affix_array, &new_subtree, max_depth, leaf_callback);
            new_subtree.pop();
        }
    }
}

/// Starts the optimization process. Calculates all possible combinations for the given chunk (subtree) of the affix tree.
/// This process is independent of the other chunks.
///
/// # Arguments
/// * `chunks` - A vector of vectors of affixes. Each chunk represents a subtree of the affix tree. The chunks are generated by the JS code and distributed to multiple web workers.
/// * `combinations` - A vector of extras combinations. To calculate the best runes and sigils we must calculate the resulting stats for each combination of extras. Also contains important optimizer settings.
pub fn start(chunks: &Vec<Vec<Affix>>, combinations: &Vec<Combination>) -> Result {
    let rankby = combinations[0].rankby;
    let mut result: Result = Result::new(rankby, combinations[0].maxResults as usize);

    let counter = RefCell::new(0);
    let mut character = Character::new(rankby);

    let max_depth = &combinations[0].slots;

    // this callback is called for every affix combination (leaf). this is where we calculate the resulting stats
    // crucuial to optimize every call in this function as it will be called millions of times
    let mut callback = |subtree: &[Affix]| {
        // Leaf callback implementation

        for setting in combinations.iter() {
            character.clear();

            test_character(&mut character, setting, subtree);

            // insert into result_characters if better than worst character
            result.insert(&character);
            *counter.borrow_mut() += 1;
        }
    };

    for chunk in chunks {
        // start dfs into tree

        // TODO
        // will need to add another for loop here to iterate over all combinations
        // there are two options for this:
        // 1. add another for loop here and iterate over all combinations; start DFS in total #combinations * #chunks times
        // 2. loop over combination in callback and start DFS only #chunks times
        // ```
        // for _ in 0..combinations.len() {
        //     descend_subtree_dfs(affix_array, &chunk, *max_depth as usize, &mut callback);
        // }
        // ```

        descend_subtree_dfs(
            &combinations[0].affixesArray,
            &chunk,
            *max_depth as usize,
            &mut callback,
        );
    }

    return result;
}

fn test_character(character: &mut Character, settings: &Combination, subtree: &[Affix]) {
    // add base attributes from settings to character
    settings.baseAttributes.iter().for_each(|(key, value)| {
        character.base_attributes.add(*key, *value);
    });

    for (index, affix) in subtree.iter().enumerate() {
        // find out stats for each affix and add them to the character
        let index_in_affix_array = settings.affixesArray[index]
            .iter()
            .position(|&r| r.to_number() == affix.to_number())
            .unwrap();
        let attributes_to_add = &settings.affixStatsArray[index][index_in_affix_array];

        // this call is expensive!
        attributes_to_add.iter().for_each(|(key, value)| {
            character.base_attributes.add(*key, *value);
        });

        character.gear[index] = *affix;
    }

    // calculate stats for the character
    update_attributes(character, settings, false);
}

fn update_attributes(character: &mut Character, settings: &Combination, no_rounding: bool) {
    calc_stats(character, settings, no_rounding);
    //print_attr(&character.attributes);

    let power_damage_score = calc_power(character, &settings);
    let condi_damage_score = 0.0;

    character.attributes.set(
        Attribute::Damage,
        power_damage_score + condi_damage_score + character.attributes.get(Attribute::FlatDPS),
    );

    // todo calcCondi

    // todo update damage

    // todo calcSurvivability
    // todo calcHealing
}

fn calc_stats(character: &mut Character, settings: &Combination, no_rounding: bool) {
    // move base attributes to attributes as default
    character.attributes = character.base_attributes.clone();

    // get references to play with
    let attributes = &mut character.attributes;
    let base_attributes = &character.base_attributes;

    // closure for rounding values depending on no_rounding
    let round = |val: f32| {
        if no_rounding {
            val
        } else {
            round_even(val)
        }
    };

    // handle convert modifiers
    for (attribute, conversion) in &settings.modifiers.convert {
        let maybe_round = |val: f32| {
            if attribute.is_point_key() {
                round(val)
            } else {
                val
            }
        };

        for (source, percent) in conversion {
            attributes.add(
                *attribute,
                maybe_round(base_attributes.get(*source) * percent),
            );
        }
    }

    // handle buff modifiers, these are simply added to the existing attributes
    for (attribute, bonus) in &settings.modifiers.buff {
        attributes.add(*attribute, *bonus);
    }

    // recalculate attributes
    attributes.add(
        Attribute::CriticalChance,
        (attributes.get(Attribute::Precision) - 1000.0) / 21.0 / 100.0,
    );
    attributes.add(
        Attribute::CriticalDamage,
        attributes.get(Attribute::Ferocity) / 15.0 / 100.0,
    );
    attributes.add(
        Attribute::BoonDuration,
        attributes.get(Attribute::Concentration) / 15.0 / 100.0,
    );
    attributes.set(
        Attribute::Health,
        round(
            (attributes.get(Attribute::Health) + attributes.get(Attribute::Vitality) * 10.0)
                * (1.0 + attributes.get(Attribute::MaxHealth)),
        ),
    );

    // clones/phantasms/shroud
    //TODO

    // handle convertAfterBuffs modifiers
    for (attribute, conversion) in &settings.modifiers.convertAfterBuffs {
        let maybe_round = |val: f32| {
            if attribute.is_point_key() {
                round(val)
            } else {
                val
            }
        };

        for (source, percent) in conversion {
            match *source {
                Attribute::CriticalChance => {
                    attributes.set(
                        Attribute::CriticalChance,
                        maybe_round(
                            clamp(attributes.get(Attribute::CriticalChance), 0.0, 1.0) * percent,
                        ),
                    );
                }
                Attribute::CloneCriticalChance => {
                    // replace macro with set
                    attributes.set(
                        Attribute::CloneCriticalChance,
                        maybe_round(
                            clamp(attributes.get(Attribute::CloneCriticalChance), 0.0, 1.0)
                                * percent,
                        ),
                    );
                }
                Attribute::PhantasmCriticalChance => {
                    attributes.set(
                        Attribute::PhantasmCriticalChance,
                        maybe_round(
                            clamp(attributes.get(Attribute::PhantasmCriticalChance), 0.0, 1.0)
                                * percent,
                        ),
                    );
                }

                _ => {
                    attributes.set(*attribute, maybe_round(attributes.get(*source) * percent));
                }
            }
        }
    }
}

pub fn calc_power(character: &mut Character, settings: &Combination) -> f32 {
    let attributes = &mut character.attributes;
    let mods = &settings.modifiers;

    let crit_dmg = attributes.get(Attribute::CriticalDamage)
        * mods.get_dmg_multiplier(Attribute::CriticalDamage);
    let crit_chance = clamp(attributes.get(Attribute::CriticalChance), 0.0, 1.0);

    attributes.set(
        Attribute::EffectivePower,
        attributes.get(Attribute::Power)
            * (1.0 + crit_chance * (crit_dmg - 1.0))
            * mods.get_dmg_multiplier(Attribute::StrikeDamage),
    );
    attributes.set(
        Attribute::NonCritEffectivePower,
        attributes.get(Attribute::Power) * mods.get_dmg_multiplier(Attribute::StrikeDamage),
    );

    // 2597: standard enemy armor value, also used for ingame damage tooltips
    let power_damage = (attributes.get(Attribute::PowerCoefficient) / 2597.0)
        * attributes.get(Attribute::EffectivePower)
        + (attributes.get(Attribute::NonCritPowerCoefficient) / 2597.0)
            * attributes.get(Attribute::NonCritEffectivePower);
    // this is nowhere read again?
    // attributes.set(Attribute::PowerDPS, power_damage);

    if attributes.get(Attribute::Power2Coefficient) > 0.0 {
        // do stuff
        //TODO implement power2 calc
    } else {
        attributes.set(Attribute::Power2DPS, 0.0);
    }

    let siphon_damage = attributes.get(Attribute::SiphonBaseCoefficient)
        * mods.get_dmg_multiplier(Attribute::SiphonDamage);
    attributes.set(
        Attribute::SiphonDPS,
        siphon_damage * attributes.get(Attribute::EffectivePower),
    );

    return power_damage + siphon_damage;
}
