# natural-bodybuilder

This is just a thing I wrote on a train ride to come up with a recipe for my new blender and avoid having to cook when arriving home.
Spaghetti code but appears to work.
Uses simple greedy search.

Usage
---

Create a `target` toml file with your desired outcome and ingredient toml files.

```
$ natural-bodybuilder -h
usage: mix target.toml ingredient0.toml ... ingredient10.toml
```

```
$ cat ./examples/target-bananana.toml
kcal = 1500
# in fractions
carb = 40
fat = 30
protein = 30

[[constraint_exact]]
name = "banana"
g = 200

[[constraint_at_most]]
name = "quark40"
g = 500

[[constraint_at_least]]
name = "oats"
g = 75
```

```
$ cat ./examples/quark40.toml 
name = "quark40"
g = 1000
kcal = 1390
fat = 100
carb = 32
protein = 90
```

```
$ natural-bodybuilder ./examples/target-bananana.toml ./examples/quark40.toml ./examples/banana.toml ./examples/seeds.toml ./examples/oats.toml
Starting search with
	Target NormalizedTarget { carb: 0.4, fat: 0.3, protein: 0.3 }
	constraints exact: None, at least: Some([TargetConstraint { name: "banana", g: 378 }]), at most Some([TargetConstraint { name: "quark40", g: 500 }, TargetConstraint { name: "seeds", g: 75 }])
	Ingredient quark40 NormalizedIngredient { carb: 0.02302158273381295, fat: 0.07194244604316546, protein: 0.06474820143884892 }
	Ingredient banana NormalizedIngredient { carb: 0.2727272727272727, fat: 0.0, protein: 0.00909090909090909 }
	Ingredient seeds NormalizedIngredient { carb: 0.009950248756218905, fat: 0.0812603648424544, protein: 0.04975124378109453 }
	Ingredient oats NormalizedIngredient { carb: 0.4223021582733813, fat: 0.050359712230215826, protein: 0.09712230215827339 }
	Found Proposal({"oats": 30, "quark40": 927, "banana": 440, "seeds": 603}) with cost 0.0026151607101754103

---- RESULT ----
Mix the following together (in grams) Proposal({"seeds": 75, "quark40": 500, "banana": 378, "oats": 16})
Results in 120g carb, 88g fat, 73g protein in 1500 kcal (43:31:26).
```
