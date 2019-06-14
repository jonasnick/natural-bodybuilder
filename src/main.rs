use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Ingredient {
    name: String,
    g: u64,
    kcal: u64,
    // in g
    carb: u64,
    // in g
    fat: u64,
    // in g
    protein: u64,
}

impl Ingredient {
    fn normalize(&self) -> NormalizedIngredient {
        let carb = self.carb as f64 / (self.kcal as f64);
        let fat = self.fat as f64 / (self.kcal as f64);
        let protein = self.protein as f64 / (self.kcal as f64);
        NormalizedIngredient {
            carb: carb,
            fat: fat,
            protein: protein,
        }
    }
}

/// carb, fat and protein in grams per kcal
#[derive(Clone, Debug)]
struct NormalizedIngredient {
    carb: f64,
    fat: f64,
    protein: f64,
}

impl NormalizedIngredient {
    fn new() -> NormalizedIngredient {
        NormalizedIngredient {
            carb: 0.0,
            fat: 0.0,
            protein: 0.0,
        }
    }
}
struct Ingredients(HashMap<String, NormalizedIngredient>);

#[derive(Clone, Debug, Eq, PartialEq)]
struct Proposal(HashMap<String, u64>);
impl Proposal {
    /// Mixes the ingredients in the proposal and returns a single normalized ingredient
    fn mix(&self, ingredients: &Ingredients) -> NormalizedIngredient {
        let mut result = NormalizedIngredient::new();
        let mut n = 0.0;
        for (name, num) in &self.0 {
            result.carb += *num as f64 * ingredients.0[name].carb;
            result.fat += *num as f64 * ingredients.0[name].fat;
            result.protein += *num as f64 * ingredients.0[name].protein;
            n += *num as f64;
        }
        result.carb /= n;
        result.fat /= n;
        result.protein /= n;

        return result;
    }
    fn kcal(&self) -> u64 {
        let mut sum = 0;
        for (_, n) in &self.0 {
            sum += n;
        }
        sum
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Target {
    kcal: u64,
    // in ratio
    carb: u64,
    // in ratio
    fat: u64,
    // in ratio
    protein: u64,
}

impl Target {
    fn normalize(&self) -> NormalizedTarget {
        NormalizedTarget {
            carb: self.carb as f64 / 100.0,
            fat: self.fat as f64 / 100.0,
            protein: self.protein as f64 / 100.0,
        }
    }
}

#[derive(Debug)]
struct NormalizedTarget {
    // in ratio
    carb: f64,
    // in ratio
    fat: f64,
    // in ratio
    protein: f64,
}

fn square(x: f64) -> f64 {
    x * x
}
impl NormalizedTarget {
    /// Using squared difference, lower is better
    fn evaluate(&self, proposal: &Proposal, ingredients: &Ingredients) -> f64 {
        let proposal_mix = proposal.mix(&ingredients);
        let sum = proposal_mix.carb + proposal_mix.fat + proposal_mix.protein;
        return square(self.carb - proposal_mix.carb/sum)
            + square(self.fat - proposal_mix.fat/sum)
            + square(self.protein - proposal_mix.protein/sum);
    }
}

fn optimize(target: &NormalizedTarget, ingredients: &Ingredients, steps: usize) -> Proposal {
    let mut proposal = Proposal(HashMap::new());
    for (name, _) in &ingredients.0 {
        proposal.0.insert(name.to_string(), 0);
    }
    for _ in 0..steps {
        let mut min_cost = None;
        let mut best_ingredient = None;
        // optimize greedily
        for (name, _) in &ingredients.0 {
            *proposal.0.get_mut(name).unwrap() += 1;
            let cost = target.evaluate(&proposal, ingredients);
            //println!("\tAdd {}, cost {}", name, cost);
            min_cost = match min_cost {
                None => {
                    best_ingredient = Some(name);
                    Some(cost)
                }
                Some(min_cost) => {
                    if cost < min_cost {
                        best_ingredient = Some(name);
                        Some(cost)
                    } else {
                        Some(min_cost)
                    }
                }
            };
            *proposal.0.get_mut(name).unwrap() -= 1;
        }
        *proposal.0.get_mut(best_ingredient.unwrap()).unwrap() += 1;
        //println!("Add {}, cost {}", best_ingredient.unwrap(), target.evaluate(&proposal, ingredients));
    }
    proposal
}

fn help() {
    println!("usage: mix target.toml ingredient0.toml ... ingredient10.toml");
}

pub fn read_file(filepath: &str) -> String {
    let file = File::open(filepath)
        .expect("could not open file");
    let mut buffered_reader = BufReader::new(file);
    let mut contents = String::new();
    let _number_of_bytes: usize = match buffered_reader.read_to_string(&mut contents) {
        Ok(number_of_bytes) => number_of_bytes,
        Err(_err) => 0
    };

    contents
}

fn main() {
    if std::env::args().len() < 3 {
        help();
        return
    }
    let target_path = std::env::args().nth(1).expect("no pattern given");
    let target: Target = toml::from_str(&read_file(&target_path)).expect("can't read target");
    let target_normalized = target.normalize();
    println!("Starting search with");
    println!("\tTarget {:?}", target_normalized);
    let mut ingredients = Ingredients(HashMap::new());
    let mut raw_ingredients = HashMap::new();
    for ingredient_path in std::env::args().skip(2) {
        let ingredient: Ingredient = toml::from_str(&read_file(&ingredient_path)).expect("can't read target");
        raw_ingredients.insert(ingredient.name.clone(), ingredient.clone());
        let normalized = ingredient.normalize();
        println!("\tIngredient {} {:?}", &ingredient.name, normalized);
        ingredients.0.insert(ingredient.name.clone(), normalized);
    }

    let proposal = optimize(&target_normalized, &ingredients, 2000);
    println!("\tFound {:?} with cost {}", proposal, target_normalized.evaluate(&proposal, &ingredients));

    // Compute grams for each ingredient because proposal is only in kcal
    let mut gram_proposal = Proposal(HashMap::new());
    let proposal_kcal = proposal.kcal();
    for (name, n) in &proposal.0 {
        let ingredient_kcal = *n as f64*(target.kcal as f64/proposal_kcal as f64);
        gram_proposal.0.insert(name.to_string(), (ingredient_kcal*(raw_ingredients[name].g as f64/raw_ingredients[name].kcal as f64)).round() as u64);
    }
    println!("");
    println!("---- RESULT ----");
    println!("Mix {:?}", gram_proposal);

    // Print macros of result
    let mut carb = 0.0;
    let mut fat = 0.0;
    let mut protein = 0.0;
    for (name, g) in &gram_proposal.0 {
        let factor = *g as f64 / raw_ingredients[name].g as f64;
        carb += factor * raw_ingredients[name].carb as f64;
        fat += factor * raw_ingredients[name].fat as f64;
        protein += factor * raw_ingredients[name].protein as f64;
    }
    let sum = carb + fat + protein;
    println!("Results in {}g carb, {}g fat, {}g protein in {} kcal ({}:{}:{}).", carb.round(), fat.round(), protein.round(), target.kcal, (100.0*carb/sum).round(), (100.0*fat/sum).round(), (100.0*protein/sum).round());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize() {
        let i = Ingredient {
            name: "foo".to_string(),
            g: 1000,
            kcal: 100,
            carb: 300,
            fat: 200,
            protein: 100,
        };
        let normalized = i.normalize();
        assert_eq!(normalized.carb.round() as u64, 3);
        assert_eq!(normalized.fat.round() as u64, 2);
        assert_eq!(normalized.protein.round() as u64, 1);
    }

    fn test_ingredients() -> Ingredients {
        let mut ingredients = Ingredients(HashMap::new());
        ingredients.0.insert(
            "apple".to_string(),
            NormalizedIngredient {
                carb: 20.0,
                fat: 30.0,
                protein: 50.0,
            },
        );
        ingredients.0.insert(
            "banana".to_string(),
            NormalizedIngredient {
                carb: 40.0,
                fat: 50.0,
                protein: 60.0,
            },
        );
        ingredients
    }

    #[test]
    fn test_mix() {
        let ingredients = test_ingredients();
        let mut proposal = Proposal(HashMap::new());
        proposal.0.insert("apple".to_string(), 1);
        let mix = proposal.mix(&ingredients);
        assert_eq!(mix.carb as u64, 20);
        assert_eq!(mix.fat as u64, 30);
        assert_eq!(mix.protein as u64, 50);

        proposal.0.clear();
        proposal.0.insert("apple".to_string(), 2);
        let mix = proposal.mix(&ingredients);
        assert_eq!(mix.carb as u64, 20);
        assert_eq!(mix.fat as u64, 30);
        assert_eq!(mix.protein as u64, 50);

        proposal.0.clear();
        proposal.0.insert("apple".to_string(), 2);
        proposal.0.insert("banana".to_string(), 1);
        let mix = proposal.mix(&ingredients);
        assert_eq!(mix.carb.round() as u64, 27);
        assert_eq!(mix.fat.round() as u64, 37);
        assert_eq!(mix.protein.round() as u64, 53);
    }

    #[test]
    fn test_evaluate() {
        let t = NormalizedTarget {
            carb: 0.20,
            fat: 0.30,
            protein: 0.50,
        };
        let ingredients = test_ingredients();
        let mut proposal = Proposal(HashMap::new());
        proposal.0.insert("apple".to_string(), 1);
        assert_eq!(t.evaluate(&proposal, &ingredients).round() as u64, 0);
        proposal.0.insert("apple".to_string(), 2);
        assert_eq!(t.evaluate(&proposal, &ingredients).round() as u64, 0);

        let t = NormalizedTarget {
            carb: 0.3,
            fat: 0.5,
            protein: 0.2,
        };
        assert_eq!(
            t.evaluate(&proposal, &ingredients),
            0.1*0.1 + 0.2*0.2 + 0.3*0.3
        );

        let t = NormalizedTarget {
            carb: 0.20,
            fat: 0.30,
            protein: 0.50,
        };
        let mut proposal = Proposal(HashMap::new());
        proposal.0.insert("banana".to_string(), 1);
        assert_eq!(t.evaluate(&proposal, &ingredients).round() as u64, 0);
    }

    #[test]
    fn test_optimize() {
        // apple target
        let t = NormalizedTarget {
            carb: 0.20,
            fat: 0.30,
            protein: 0.50,
        };
        let ingredients = test_ingredients();
        let proposal = optimize(&t, &ingredients, 2);
        let mut expected_proposal = Proposal(HashMap::new());
        expected_proposal.0.insert("apple".to_string(), 2);
        expected_proposal.0.insert("banana".to_string(), 0);
        assert_eq!(proposal, expected_proposal);

        // banana target
        let t = NormalizedTarget {
            carb: 0.26,
            fat: 0.33,
            protein: 0.4,
        };
        let ingredients = test_ingredients();
        let proposal = optimize(&t, &ingredients, 2);
        let mut expected_proposal = Proposal(HashMap::new());
        expected_proposal.0.insert("apple".to_string(), 0);
        expected_proposal.0.insert("banana".to_string(), 2);
        assert_eq!(proposal, expected_proposal);

        let t = NormalizedTarget {
            carb: 0.23,
            fat: 0.315,
            protein: 0.45,
        };
        let ingredients = test_ingredients();
        let proposal = optimize(&t, &ingredients, 2);
        let mut expected_proposal = Proposal(HashMap::new());
        expected_proposal.0.insert("apple".to_string(), 1);
        expected_proposal.0.insert("banana".to_string(), 1);
        assert_eq!(proposal, expected_proposal);
    }
}
