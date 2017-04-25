# factorio-recipes-planner
Planning tool for building factories in Factorio, written in Rust.

Consists of two main parts:

## Fetcher
Fetcher get the Lua scripts with data from Factorio data folders and
parses it, using Nom. Then, it transforms it to the appropriate form and
save it to "new.data", storing some statistics alongside.
So actually it is parser of Lua tables in pure Rust.

## Planner
Planner takes data, gathered by the fetcher and build a dependency graph
from recipes. Than it takes a desired component we want to produce and
desired throughput and calculate the exact number of factories we need
to fullfill the requirements.

## Example
Output of the planner if we want to obtain "Electronic circuit" with 
performance 1 item/second:
```
Components:
    copper-cable
    copper-ore
    electronic-circuit
    copper-plate
    iron-plate
    iron-ore
Edges:
    electronic-circuit --1--> iron-plate
    electronic-circuit --3--> copper-cable
    iron-plate --1--> iron-ore
    copper-cable --0.5--> copper-plate
    copper-plate --1--> copper-ore
Assemble plan:
    electronic-circuit: (time = 0.6666667) * (rate = 1 (60 parts/min)) = (count = 0.6666667)
    copper-cable: (time = 0.33333334) * (rate = 3 (180 parts/min)) = (count = 1)
    copper-plate: (time = 4.6666665) * (rate = 1.5 (90 parts/min)) = (count = 7)
    iron-plate: (time = 4.6666665) * (rate = 1 (60 parts/min)) = (count = 4.6666665)
Components flow rate:
    copper-ore: rate = 90 parts/min
    iron-ore: rate = 60 parts/min
```

So we need 1 factory for circuit, 1 for copper cable, 7 furnaces for copper plate and 5 furnaces for iron plate.
Eventually we need 90 pieces of copper ore per minute and 60 pieces of iron ore.
