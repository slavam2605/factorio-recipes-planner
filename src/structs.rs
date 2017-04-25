#[derive(Debug)]
pub struct RecipeComponent {
    pub a_type: String,
    pub name: String,
    pub amount: f64
}

#[derive(Debug)]
pub struct Recipe {
    pub name: String,
    pub category: String,
    pub products: Vec<RecipeComponent>,
    pub ingredients: Vec<RecipeComponent>,
    pub energy_required: f64,
    pub enabled: bool
}

#[derive(Debug)]
pub enum Power {
    Burner(i64),
    Electric(i64)
}

use Power::*;

pub struct TransformMachine {
    pub name: &'static str,
    pub energy_consumption: Power,
    pub crafting_speed: f64,
    pub module_slots: i32,
    pub pollution: f64
}

pub struct MiningMachine {
    pub name: &'static str,
    pub energy_consumption: Power,
    pub mining_speed: f64,
    pub mining_power: f64,
    pub module_slots: i32,
    pub pollution: f64
}

pub trait Machine {
    fn name(&self) -> String;
    fn energy_consumption(&self) -> &Power;
    fn module_slots(&self) -> i32;
    fn pollution(&self) -> f64;
}

impl Machine for TransformMachine {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn energy_consumption(&self) -> &Power {
        &self.energy_consumption
    }

    fn module_slots(&self) -> i32 {
        self.module_slots
    }

    fn pollution(&self) -> f64 {
        self.pollution
    }
}

impl Machine for MiningMachine {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn energy_consumption(&self) -> &Power {
        &self.energy_consumption
    }

    fn module_slots(&self) -> i32 {
        self.module_slots
    }

    fn pollution(&self) -> f64 {
        self.pollution
    }
}

pub const CHEMICAL_PLANT: TransformMachine = TransformMachine {
    name: "chemical-plant",
    energy_consumption: Electric(210_000),
    crafting_speed: 1.25,
    module_slots: 2,
    pollution: 1.8
};

pub const ELECTRIC_FURNACE: TransformMachine = TransformMachine {
    name: "electric-furnace",
    energy_consumption: Electric(180_000),
    crafting_speed: 2.0,
    module_slots: 2,
    pollution: 0.9
};

pub const STEEL_FURNACE: TransformMachine = TransformMachine {
    name: "steel-furnace",
    energy_consumption: Burner(180_000),
    crafting_speed: 2.0,
    module_slots: 0,
    pollution: 3.6
};

pub const STONE_FURNACE: TransformMachine = TransformMachine {
    name: "stone-furnace",
    energy_consumption: Burner(180_000),
    crafting_speed: 1.0,
    module_slots: 0,
    pollution: 1.8
};

pub const ASSEMBLING_MACHINE_1: TransformMachine = TransformMachine {
    name: "assembling-machine-1",
    energy_consumption: Electric(90_000),
    crafting_speed: 0.5,
    module_slots: 0,
    pollution: 3.0
};

pub const ASSEMBLING_MACHINE_2: TransformMachine = TransformMachine {
    name: "assembling-machine-2",
    energy_consumption: Electric(150_000),
    crafting_speed: 0.75,
    module_slots: 2,
    pollution: 2.4
};

pub const ASSEMBLING_MACHINE_3: TransformMachine = TransformMachine {
    name: "assembling-machine-3",
    energy_consumption: Electric(210_000),
    crafting_speed: 1.25,
    module_slots: 4,
    pollution: 1.8
};

pub const OIL_REFINERY: TransformMachine = TransformMachine {
    name: "oil-refinery",
    energy_consumption: Electric(420_000),
    crafting_speed: 1.0,
    module_slots: 2,
    pollution: 3.6
};

pub const ELECTRIC_MINING_DRILL: MiningMachine = MiningMachine {
    name: "electric-mining-drill",
    energy_consumption: Electric(90_000),
    mining_speed: 0.5,
    mining_power: 3.0,
    module_slots: 3,
    pollution: 9.0
};

pub const BURNER_MINING_DRILL: MiningMachine = MiningMachine {
    name: "burner-mining-drill",
    energy_consumption: Burner(300_000),
    mining_speed: 0.35,
    mining_power: 2.5,
    module_slots: 0,
    pollution: 10.0
}; 

pub const PUMPJACK: MiningMachine = MiningMachine {
    name: "pumpjack",
    energy_consumption: Electric(90_000),
    mining_speed: 1.0,
    mining_power: 2.0,
    module_slots: 2,
    pollution: 9.0
};